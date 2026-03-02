//! A windowing shell for Iced, on top of [`winit`].
//!
//! ![The native path of the Iced ecosystem](https://github.com/iced-rs/iced/blob/0525d76ff94e828b7b21634fa94a747022001c83/docs/graphs/native.png?raw=true)
//!
//! `iced_winit` offers some convenient abstractions on top of [`iced_runtime`]
//! to quickstart development when using [`winit`].
//!
//! It exposes a renderer-agnostic [`Program`] trait that can be implemented
//! and then run with a simple call. The use of this trait is optional.
//!
//! Additionally, a [`conversion`] module is available for users that decide to
//! implement a custom event loop.
//!
//! [`iced_runtime`]: https://github.com/iced-rs/iced/tree/master/runtime
//! [`winit`]: https://github.com/rust-windowing/winit
//! [`conversion`]: crate::conversion
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
pub use iced_debug as debug;
pub use iced_program as program;
pub use iced_runtime as runtime;
pub use program::core;
pub use program::graphics;
pub use runtime::futures;
pub use winit;

pub mod clipboard;
pub mod conversion;

mod error;
mod proxy;
mod window;

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "ios", target_os = "android"))
))]
mod popup;

pub use clipboard::Clipboard;
pub use error::Error;
pub use proxy::Proxy;

use crate::core::mouse;
use crate::core::renderer;
use crate::core::theme;
use crate::core::time::Instant;
use crate::core::widget::operation;
use crate::core::{Point, Size};
use crate::futures::futures::channel::mpsc;
use crate::futures::futures::channel::oneshot;
use crate::futures::futures::task;
use crate::futures::futures::{Future, StreamExt};
use crate::futures::subscription;
use crate::futures::{Executor, Runtime};
use crate::graphics::{Compositor, Shell, compositor};
use crate::runtime::image;
use crate::runtime::system;
use crate::runtime::user_interface::{self, UserInterface};
use crate::runtime::{Action, Task};

use program::Program;
use window::WindowManager;

use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::mem::ManuallyDrop;
use std::slice;
use std::sync::Arc;

/// Runs a [`Program`] with the provided settings.
pub fn run<P>(program: P) -> Result<(), Error>
where
    P: Program + 'static,
    P::Theme: theme::Base,
{
    use winit::event_loop::EventLoop;

    let boot_span = debug::boot();
    let settings = program.settings();
    let window_settings = program.window();

    let event_loop = EventLoop::with_user_event()
        .build()
        .expect("Create event loop");

    let graphics_settings = settings.clone().into();
    let display_handle = event_loop.owned_display_handle();

    let (proxy, worker) = Proxy::new(event_loop.create_proxy());

    #[cfg(feature = "debug")]
    {
        let proxy = proxy.clone();

        debug::on_hotpatch(move || {
            proxy.send_action(Action::Reload);
        });
    }

    let mut runtime = {
        let executor = P::Executor::new().map_err(Error::ExecutorCreationFailed)?;
        executor.spawn(worker);

        Runtime::new(executor, proxy.clone())
    };

    let (program, task) = runtime.enter(|| program::Instance::new(program));
    let is_daemon = window_settings.is_none();

    let task = if let Some(window_settings) = window_settings {
        let mut task = Some(task);

        let (_id, open) = runtime::window::open(window_settings);

        open.then(move |_| task.take().unwrap_or_else(Task::none))
    } else {
        task
    };

    if let Some(stream) = runtime::task::into_stream(task) {
        runtime.run(stream);
    }

    runtime.track(subscription::into_recipes(
        runtime.enter(|| program.subscription().map(Action::Output)),
    ));

    let (event_sender, event_receiver) = mpsc::unbounded();
    let (control_sender, control_receiver) = mpsc::unbounded();
    let (system_theme_sender, system_theme_receiver) = oneshot::channel();

    let instance = Box::pin(run_instance::<P>(
        program,
        runtime,
        proxy.clone(),
        event_receiver,
        control_sender,
        display_handle,
        is_daemon,
        graphics_settings,
        settings.fonts,
        system_theme_receiver,
    ));

    let context = task::Context::from_waker(task::noop_waker_ref());

    struct Runner<Message: 'static, F> {
        instance: std::pin::Pin<Box<F>>,
        context: task::Context<'static>,
        id: Option<String>,
        sender: mpsc::UnboundedSender<Event<Action<Message>>>,
        receiver: mpsc::UnboundedReceiver<Control>,
        error: Option<Error>,
        system_theme: Option<oneshot::Sender<theme::Mode>>,

        #[cfg(target_arch = "wasm32")]
        canvas: Option<web_sys::HtmlCanvasElement>,
    }

    let runner = Runner {
        instance,
        context,
        id: settings.id,
        sender: event_sender,
        receiver: control_receiver,
        error: None,
        system_theme: Some(system_theme_sender),

        #[cfg(target_arch = "wasm32")]
        canvas: None,
    };

    boot_span.finish();

    impl<Message, F> winit::application::ApplicationHandler<Action<Message>> for Runner<Message, F>
    where
        F: Future<Output = ()>,
    {
        fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
            if let Some(sender) = self.system_theme.take() {
                let _ = sender.send(
                    event_loop
                        .system_theme()
                        .map(conversion::theme_mode)
                        .unwrap_or_default(),
                );
            }
        }

        fn new_events(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            cause: winit::event::StartCause,
        ) {
            self.process_event(
                event_loop,
                Event::EventLoopAwakened(winit::event::Event::NewEvents(cause)),
            );
        }

        fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            window_id: winit::window::WindowId,
            event: winit::event::WindowEvent,
        ) {
            #[cfg(target_os = "windows")]
            let is_move_or_resize = matches!(
                event,
                winit::event::WindowEvent::Resized(_) | winit::event::WindowEvent::Moved(_)
            );

            self.process_event(
                event_loop,
                Event::EventLoopAwakened(winit::event::Event::WindowEvent { window_id, event }),
            );

            // TODO: Remove when unnecessary
            // On Windows, we emulate an `AboutToWait` event after every `Resized` event
            // since the event loop does not resume during resize interaction.
            // More details: https://github.com/rust-windowing/winit/issues/3272
            #[cfg(target_os = "windows")]
            {
                if is_move_or_resize {
                    self.process_event(
                        event_loop,
                        Event::EventLoopAwakened(winit::event::Event::AboutToWait),
                    );
                }
            }
        }

        fn user_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            action: Action<Message>,
        ) {
            self.process_event(
                event_loop,
                Event::EventLoopAwakened(winit::event::Event::UserEvent(action)),
            );
        }

        fn received_url(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, url: String) {
            self.process_event(
                event_loop,
                Event::EventLoopAwakened(winit::event::Event::PlatformSpecific(
                    winit::event::PlatformSpecific::MacOS(winit::event::MacOS::ReceivedUrl(url)),
                )),
            );
        }

        fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
            // Poll popup events from winit BEFORE processing AboutToWait,
            // so they're in the event channel when AboutToWait dispatches events.
            #[cfg(all(
                unix,
                not(any(target_os = "macos", target_os = "ios", target_os = "android"))
            ))]
            {
                use winit::platform::wayland::ActiveEventLoopExtWayland;

                for popup_event in event_loop.take_popup_events() {
                    match popup_event {
                        winit::platform::wayland::PopupEvent::Configure { id, width, height } => {
                            // Get raw handles for the popup surface
                            if let Some((surface_ptr, display_ptr)) =
                                event_loop.popup_raw_handles(id)
                            {
                                let _ = self.sender.unbounded_send(Event::PopupConfigured {
                                    winit_popup_id: id.0,
                                    width,
                                    height,
                                    surface_ptr,
                                    display_ptr,
                                });
                            }
                        }
                        winit::platform::wayland::PopupEvent::Done { id } => {
                            let _ = self.sender.unbounded_send(Event::PopupClosed {
                                winit_popup_id: id.0,
                            });
                        }
                        winit::platform::wayland::PopupEvent::PointerEnter { id, x, y } => {
                            let _ = self.sender.unbounded_send(Event::PopupPointerEvent {
                                winit_popup_id: id.0,
                                kind: PopupPointerEventKind::Enter { x, y },
                            });
                        }
                        winit::platform::wayland::PopupEvent::PointerLeave { id } => {
                            let _ = self.sender.unbounded_send(Event::PopupPointerEvent {
                                winit_popup_id: id.0,
                                kind: PopupPointerEventKind::Leave,
                            });
                        }
                        winit::platform::wayland::PopupEvent::PointerMotion { id, x, y } => {
                            let _ = self.sender.unbounded_send(Event::PopupPointerEvent {
                                winit_popup_id: id.0,
                                kind: PopupPointerEventKind::Motion { x, y },
                            });
                        }
                        winit::platform::wayland::PopupEvent::PointerButton {
                            id,
                            button,
                            pressed,
                        } => {
                            let _ = self.sender.unbounded_send(Event::PopupPointerEvent {
                                winit_popup_id: id.0,
                                kind: PopupPointerEventKind::Button { button, pressed },
                            });
                        }
                    }
                }
            }

            self.process_event(
                event_loop,
                Event::EventLoopAwakened(winit::event::Event::AboutToWait),
            );
        }
    }

    impl<Message, F> Runner<Message, F>
    where
        F: Future<Output = ()>,
    {
        fn process_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            event: Event<Action<Message>>,
        ) {
            if event_loop.exiting() {
                return;
            }

            self.sender.start_send(event).expect("Send event");

            loop {
                let poll = self.instance.as_mut().poll(&mut self.context);

                match poll {
                    task::Poll::Pending => match self.receiver.try_recv() {
                        Ok(control) => match control {
                            Control::ChangeFlow(flow) => {
                                use winit::event_loop::ControlFlow;

                                match (event_loop.control_flow(), flow) {
                                    (
                                        ControlFlow::WaitUntil(current),
                                        ControlFlow::WaitUntil(new),
                                    ) if current < new => {}
                                    (ControlFlow::WaitUntil(target), ControlFlow::Wait)
                                        if target > Instant::now() => {}
                                    _ => {
                                        event_loop.set_control_flow(flow);
                                    }
                                }
                            }
                            Control::CreateWindow {
                                id,
                                settings,
                                title,
                                scale_factor,
                                monitor,
                                on_open,
                            } => {
                                let exit_on_close_request = settings.exit_on_close_request;

                                let visible = settings.visible;

                                #[cfg(target_arch = "wasm32")]
                                let target = settings.platform_specific.target.clone();

                                let window_attributes = conversion::window_attributes(
                                    settings,
                                    &title,
                                    scale_factor,
                                    monitor.or(event_loop.primary_monitor()),
                                    self.id.clone(),
                                )
                                .with_visible(false);

                                #[cfg(target_arch = "wasm32")]
                                let window_attributes = {
                                    use winit::platform::web::WindowAttributesExtWebSys;
                                    window_attributes.with_canvas(self.canvas.take())
                                };

                                log::info!(
                                    "Window attributes for id `{id:#?}`: {window_attributes:#?}"
                                );

                                // On macOS, the `position` in `WindowAttributes` represents the "inner"
                                // position of the window; while on other platforms it's the "outer" position.
                                // We fix the inconsistency on macOS by positioning the window after creation.
                                #[cfg(target_os = "macos")]
                                let mut window_attributes = window_attributes;

                                #[cfg(target_os = "macos")]
                                let position = window_attributes.position.take();

                                let window = event_loop
                                    .create_window(window_attributes)
                                    .expect("Create window");

                                #[cfg(target_os = "macos")]
                                if let Some(position) = position {
                                    window.set_outer_position(position);
                                }

                                #[cfg(target_arch = "wasm32")]
                                {
                                    use winit::platform::web::WindowExtWebSys;

                                    let canvas = window.canvas().expect("Get window canvas");

                                    let _ = canvas.set_attribute(
                                        "style",
                                        "display: block; width: 100%; height: 100%",
                                    );

                                    let window = web_sys::window().unwrap();
                                    let document = window.document().unwrap();
                                    let body = document.body().unwrap();

                                    let target = target.and_then(|target| {
                                        body.query_selector(&format!("#{target}"))
                                            .ok()
                                            .unwrap_or(None)
                                    });

                                    match target {
                                        Some(node) => {
                                            let _ = node.replace_with_with_node_1(&canvas).expect(
                                                &format!("Could not replace #{}", node.id()),
                                            );
                                        }
                                        None => {
                                            let _ = body
                                                .append_child(&canvas)
                                                .expect("Append canvas to HTML body");
                                        }
                                    };
                                }

                                self.process_event(
                                    event_loop,
                                    Event::WindowCreated {
                                        id,
                                        window: Arc::new(window),
                                        exit_on_close_request,
                                        make_visible: visible,
                                        on_open,
                                    },
                                );
                            }
                            Control::Exit => {
                                self.process_event(event_loop, Event::Exit);
                                event_loop.exit();
                                break;
                            }
                            Control::Crash(error) => {
                                self.error = Some(error);
                                event_loop.exit();
                            }
                            #[cfg(all(
                                unix,
                                not(any(
                                    target_os = "macos",
                                    target_os = "ios",
                                    target_os = "android"
                                ))
                            ))]
                            Control::CreatePopup {
                                id,
                                parent_iced_id,
                                parent_winit_id,
                                size,
                                anchor_rect,
                                anchor,
                                gravity,
                                offset,
                                constraint_adjustment,
                                grab,
                                window_geometry,
                            } => {
                                use winit::platform::wayland::ActiveEventLoopExtWayland;
                                use winit::platform::wayland::{
                                    PopupAnchor, PopupGravity, PopupSettings as WinitPopupSettings,
                                };

                                let settings = WinitPopupSettings {
                                    parent_id: parent_winit_id,
                                    size,
                                    anchor_rect,
                                    anchor: PopupAnchor::from(anchor),
                                    gravity: PopupGravity::from(gravity),
                                    offset,
                                    constraint_adjustment,
                                    grab,
                                    window_geometry,
                                };

                                if let Some(winit_popup_id) = event_loop.create_popup(settings) {
                                    // Send event to run_instance to track the popup
                                    let _ = self.sender.unbounded_send(Event::PopupCreated {
                                        iced_id: id,
                                        winit_popup_id: winit_popup_id.0,
                                        parent_id: parent_iced_id,
                                        size,
                                    });
                                } else {
                                    tracing::warn!(
                                        "Failed to create xdg_popup for window {:?}",
                                        id
                                    );
                                }
                            }
                            #[cfg(all(
                                unix,
                                not(any(
                                    target_os = "macos",
                                    target_os = "ios",
                                    target_os = "android"
                                ))
                            ))]
                            Control::DestroyPopup { winit_popup_id } => {
                                use winit::platform::wayland::ActiveEventLoopExtWayland;
                                use winit::platform::wayland::PopupId as WinitPopupId;

                                let popup_id = WinitPopupId(winit_popup_id);
                                let _ = event_loop.destroy_popup(popup_id);
                            }
                            #[cfg(all(
                                unix,
                                not(any(
                                    target_os = "macos",
                                    target_os = "ios",
                                    target_os = "android"
                                ))
                            ))]
                            Control::ResizePopup {
                                winit_popup_id,
                                width,
                                height,
                            } => {
                                use winit::platform::wayland::ActiveEventLoopExtWayland;
                                use winit::platform::wayland::PopupId as WinitPopupId;

                                let popup_id = WinitPopupId(winit_popup_id);
                                let _ = event_loop.resize_popup(popup_id, width, height);
                            }
                            Control::SetAutomaticWindowTabbing(_enabled) => {
                                #[cfg(target_os = "macos")]
                                {
                                    use winit::platform::macos::ActiveEventLoopExtMacOS;
                                    event_loop.set_allows_automatic_window_tabbing(_enabled);
                                }
                            }
                        },
                        _ => {
                            break;
                        }
                    },
                    task::Poll::Ready(_) => {
                        event_loop.exit();
                        break;
                    }
                };
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut runner = runner;
        let _ = event_loop.run_app(&mut runner);

        runner.error.map(Err).unwrap_or(Ok(()))
    }

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;
        let _ = event_loop.spawn_app(runner);

        Ok(())
    }
}

/// Kind of pointer event on a popup surface.
#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "ios", target_os = "android"))
))]
#[derive(Debug)]
enum PopupPointerEventKind {
    Enter { x: f64, y: f64 },
    Leave,
    Motion { x: f64, y: f64 },
    Button { button: u32, pressed: bool },
}

#[derive(Debug)]
enum Event<Message: 'static> {
    WindowCreated {
        id: window::Id,
        window: Arc<winit::window::Window>,
        exit_on_close_request: bool,
        make_visible: bool,
        on_open: oneshot::Sender<window::Id>,
    },
    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ))]
    PopupCreated {
        iced_id: window::Id,
        winit_popup_id: u64,
        parent_id: window::Id,
        size: (u32, u32),
    },
    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ))]
    PopupConfigured {
        winit_popup_id: u64,
        width: u32,
        height: u32,
        surface_ptr: std::ptr::NonNull<std::ffi::c_void>,
        display_ptr: std::ptr::NonNull<std::ffi::c_void>,
    },
    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ))]
    PopupClosed {
        winit_popup_id: u64,
    },
    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ))]
    PopupPointerEvent {
        winit_popup_id: u64,
        kind: PopupPointerEventKind,
    },
    EventLoopAwakened(winit::event::Event<Message>),
    Exit,
}

#[derive(Debug)]
enum Control {
    ChangeFlow(winit::event_loop::ControlFlow),
    Exit,
    Crash(Error),
    CreateWindow {
        id: window::Id,
        settings: window::Settings,
        title: String,
        monitor: Option<winit::monitor::MonitorHandle>,
        on_open: oneshot::Sender<window::Id>,
        scale_factor: f32,
    },
    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ))]
    CreatePopup {
        id: window::Id,
        parent_iced_id: window::Id,
        parent_winit_id: winit::window::WindowId,
        size: (u32, u32),
        anchor_rect: (i32, i32, i32, i32),
        anchor: u32,
        gravity: u32,
        offset: (i32, i32),
        constraint_adjustment: u32,
        grab: bool,
        window_geometry: Option<(i32, i32, i32, i32)>,
    },
    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ))]
    DestroyPopup {
        /// The winit popup ID to destroy.
        winit_popup_id: u64,
    },
    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ))]
    ResizePopup {
        winit_popup_id: u64,
        width: u32,
        height: u32,
    },
    SetAutomaticWindowTabbing(bool),
}

async fn run_instance<P>(
    mut program: program::Instance<P>,
    mut runtime: Runtime<P::Executor, Proxy<P::Message>, Action<P::Message>>,
    mut proxy: Proxy<P::Message>,
    mut event_receiver: mpsc::UnboundedReceiver<Event<Action<P::Message>>>,
    mut control_sender: mpsc::UnboundedSender<Control>,
    display_handle: winit::event_loop::OwnedDisplayHandle,
    is_daemon: bool,
    graphics_settings: graphics::Settings,
    default_fonts: Vec<Cow<'static, [u8]>>,
    mut _system_theme: oneshot::Receiver<theme::Mode>,
) where
    P: Program + 'static,
    P::Theme: theme::Base,
{
    use crate::core::Renderer as _;
    use winit::event;
    use winit::event_loop::ControlFlow;

    let mut window_manager = WindowManager::new();
    let mut is_window_opening = !is_daemon;

    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ))]
    let mut popup_manager: popup::PopupManager<
        P,
        <P::Renderer as compositor::Default>::Compositor,
    > = popup::PopupManager::new();

    // Track cursor position per popup for event dispatch
    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ))]
    let mut popup_cursor_position: FxHashMap<window::Id, core::Point> = FxHashMap::default();

    let mut compositor = None;
    let mut events = Vec::new();
    let mut messages = Vec::new();
    let mut actions = 0;

    let mut ui_caches = FxHashMap::default();
    let mut user_interfaces = ManuallyDrop::new(FxHashMap::default());
    let mut clipboard = Clipboard::new();

    #[cfg(all(feature = "linux-theme-detection", target_os = "linux"))]
    let mut system_theme = {
        let to_mode = |color_scheme| match color_scheme {
            mundy::ColorScheme::NoPreference => theme::Mode::None,
            mundy::ColorScheme::Light => theme::Mode::Light,
            mundy::ColorScheme::Dark => theme::Mode::Dark,
        };

        runtime.run(
            mundy::Preferences::stream(mundy::Interest::ColorScheme)
                .map(move |preferences| {
                    Action::System(system::Action::NotifyTheme(to_mode(
                        preferences.color_scheme,
                    )))
                })
                .boxed(),
        );

        runtime
            .enter(|| {
                mundy::Preferences::once_blocking(
                    mundy::Interest::ColorScheme,
                    core::time::Duration::from_millis(200),
                )
            })
            .map(|preferences| to_mode(preferences.color_scheme))
            .unwrap_or_default()
    };

    #[cfg(not(all(feature = "linux-theme-detection", target_os = "linux")))]
    let mut system_theme = _system_theme.try_recv().ok().flatten().unwrap_or_default();

    log::info!("System theme: {system_theme:?}");

    'next_event: loop {
        // Empty the queue if possible
        let event = if let Ok(event) = event_receiver.try_recv() {
            Some(event)
        } else {
            event_receiver.next().await
        };

        let Some(event) = event else {
            break;
        };

        match event {
            Event::WindowCreated {
                id,
                window,
                exit_on_close_request,
                make_visible,
                on_open,
            } => {
                if compositor.is_none() {
                    let (compositor_sender, compositor_receiver) = oneshot::channel();

                    let create_compositor = {
                        let window = window.clone();
                        let display_handle = display_handle.clone();
                        let proxy = proxy.clone();
                        let default_fonts = default_fonts.clone();

                        async move {
                            let shell = Shell::new(proxy.clone());

                            let mut compositor =
                                <P::Renderer as compositor::Default>::Compositor::new(
                                    graphics_settings,
                                    display_handle,
                                    window,
                                    shell,
                                )
                                .await;

                            if let Ok(compositor) = &mut compositor {
                                for font in default_fonts {
                                    compositor.load_font(font.clone());
                                }
                            }

                            compositor_sender
                                .send(compositor)
                                .ok()
                                .expect("Send compositor");

                            // HACK! Send a proxy event on completion to trigger
                            // a runtime re-poll
                            // TODO: Send compositor through proxy (?)
                            {
                                let (sender, _receiver) = oneshot::channel();

                                proxy.send_action(Action::Window(
                                    runtime::window::Action::GetLatest(sender),
                                ));
                            }
                        }
                    };

                    #[cfg(target_arch = "wasm32")]
                    wasm_bindgen_futures::spawn_local(create_compositor);

                    #[cfg(not(target_arch = "wasm32"))]
                    runtime.block_on(create_compositor);

                    match compositor_receiver.await.expect("Wait for compositor") {
                        Ok(new_compositor) => {
                            compositor = Some(new_compositor);
                        }
                        Err(error) => {
                            let _ = control_sender.start_send(Control::Crash(error.into()));
                            continue;
                        }
                    }
                }

                let window_theme = window
                    .theme()
                    .map(conversion::theme_mode)
                    .unwrap_or_default();

                if system_theme != window_theme {
                    system_theme = window_theme;

                    runtime.broadcast(subscription::Event::SystemThemeChanged(window_theme));
                }

                let is_first = window_manager.is_empty();
                let window = window_manager.insert(
                    id,
                    window,
                    &program,
                    compositor.as_mut().expect("Compositor must be initialized"),
                    exit_on_close_request,
                    system_theme,
                );

                window
                    .raw
                    .set_theme(conversion::window_theme(window.state.theme_mode()));

                debug::theme_changed(|| {
                    if is_first {
                        theme::Base::palette(window.state.theme())
                    } else {
                        None
                    }
                });

                let logical_size = window.state.logical_size();

                #[cfg(feature = "hinting")]
                window.renderer.hint(window.state.scale_factor());

                let _ = user_interfaces.insert(
                    id,
                    build_user_interface(
                        &program,
                        user_interface::Cache::default(),
                        &mut window.renderer,
                        logical_size,
                        id,
                    ),
                );
                let _ = ui_caches.insert(id, user_interface::Cache::default());

                if make_visible {
                    window.raw.set_visible(true);
                }

                events.push((
                    id,
                    core::Event::Window(window::Event::Opened {
                        position: window.position(),
                        size: window.logical_size(),
                    }),
                ));

                let _ = on_open.send(id);
                is_window_opening = false;
            }
            #[cfg(all(
                unix,
                not(any(target_os = "macos", target_os = "ios", target_os = "android"))
            ))]
            Event::PopupCreated {
                iced_id,
                winit_popup_id,
                parent_id,
                size,
            } => {
                // Get scale factor from parent window
                let scale_factor = window_manager
                    .get(parent_id)
                    .map(|w| w.state.scale_factor())
                    .unwrap_or(1.0);

                // Store the popup in our popup manager (not yet configured)
                popup_manager.insert(
                    popup::PopupId(winit_popup_id),
                    iced_id,
                    parent_id,
                    Size::new(size.0, size.1),
                    scale_factor,
                );

                is_window_opening = false;
            }
            #[cfg(all(
                unix,
                not(any(target_os = "macos", target_os = "ios", target_os = "android"))
            ))]
            Event::PopupConfigured {
                winit_popup_id,
                width,
                height,
                surface_ptr,
                display_ptr,
            } => {
                // Create PopupSurface wrapper for compositor
                let popup_surface = popup::PopupSurface::new(surface_ptr, display_ptr);

                // Configure the popup with the compositor surface
                if let Some(comp) = compositor.as_mut() {
                    let popup_id = popup::PopupId(winit_popup_id);
                    let _ = popup_manager.configure(
                        popup_id,
                        winit_popup_id,
                        width,
                        height,
                        popup_surface,
                        comp,
                    );
                }
            }
            #[cfg(all(
                unix,
                not(any(target_os = "macos", target_os = "ios", target_os = "android"))
            ))]
            Event::PopupClosed { winit_popup_id } => {
                // Remove from popup manager and notify the application
                if let Some(removed) = popup_manager.remove(popup::PopupId(winit_popup_id)) {
                    let iced_id = removed.iced_id;
                    let _ = ui_caches.remove(&iced_id);
                    let _ = user_interfaces.remove(&iced_id);
                    let _ = popup_cursor_position.remove(&iced_id);
                    events.push((iced_id, core::Event::Window(core::window::Event::Closed)));
                }
            }
            #[cfg(all(
                unix,
                not(any(target_os = "macos", target_os = "ios", target_os = "android"))
            ))]
            Event::PopupPointerEvent {
                winit_popup_id,
                kind,
            } => {
                if let Some(popup) = popup_manager.find_by_winit_id(winit_popup_id) {
                    let iced_id = popup.iced_id;
                    let parent_id = popup.parent_id;

                    match kind {
                        PopupPointerEventKind::Enter { x, y }
                        | PopupPointerEventKind::Motion { x, y } => {
                            let position = core::Point::new(x as f32, y as f32);
                            // Store cursor position for the popup
                            let _ = popup_cursor_position.insert(iced_id, position);
                            events.push((
                                iced_id,
                                core::Event::Mouse(core::mouse::Event::CursorMoved { position }),
                            ));
                        }
                        PopupPointerEventKind::Leave => {
                            let _ = popup_cursor_position.remove(&iced_id);
                            events.push((
                                iced_id,
                                core::Event::Mouse(core::mouse::Event::CursorLeft),
                            ));
                        }
                        PopupPointerEventKind::Button { button, pressed } => {
                            // Convert Linux evdev button codes to iced mouse button
                            let mouse_button = match button {
                                0x110 => core::mouse::Button::Left,
                                0x111 => core::mouse::Button::Right,
                                0x112 => core::mouse::Button::Middle,
                                0x113 | 0x116 => core::mouse::Button::Back,
                                0x114 | 0x115 => core::mouse::Button::Forward,
                                other => core::mouse::Button::Other(other as u16),
                            };
                            let event = if pressed {
                                core::mouse::Event::ButtonPressed(mouse_button)
                            } else {
                                core::mouse::Event::ButtonReleased(mouse_button)
                            };
                            events.push((iced_id, core::Event::Mouse(event)));
                        }
                    }

                    // Request redraw on parent window to trigger popup rendering
                    if let Some((_id, window)) =
                        window_manager.iter_mut().find(|(id, _)| *id == parent_id)
                    {
                        window.raw.request_redraw();
                    }
                }
            }
            Event::EventLoopAwakened(event) => {
                match event {
                    event::Event::NewEvents(event::StartCause::Init) => {
                        for (_id, window) in window_manager.iter_mut() {
                            window.raw.request_redraw();
                        }
                    }
                    event::Event::NewEvents(event::StartCause::ResumeTimeReached { .. }) => {
                        let now = Instant::now();

                        for (_id, window) in window_manager.iter_mut() {
                            if let Some(redraw_at) = window.redraw_at
                                && redraw_at <= now
                            {
                                window.raw.request_redraw();
                                window.redraw_at = None;
                            }
                        }

                        if let Some(redraw_at) = window_manager.redraw_at() {
                            let _ = control_sender
                                .start_send(Control::ChangeFlow(ControlFlow::WaitUntil(redraw_at)));
                        } else {
                            let _ =
                                control_sender.start_send(Control::ChangeFlow(ControlFlow::Wait));
                        }
                    }
                    event::Event::PlatformSpecific(event::PlatformSpecific::MacOS(
                        event::MacOS::ReceivedUrl(url),
                    )) => {
                        runtime.broadcast(subscription::Event::PlatformSpecific(
                            subscription::PlatformSpecific::MacOS(
                                subscription::MacOS::ReceivedUrl(url),
                            ),
                        ));
                    }
                    event::Event::UserEvent(action) => {
                        run_action(
                            action,
                            &program,
                            &mut runtime,
                            &mut compositor,
                            &mut events,
                            &mut messages,
                            &mut clipboard,
                            &mut control_sender,
                            &mut user_interfaces,
                            &mut window_manager,
                            &mut ui_caches,
                            &mut is_window_opening,
                            &mut system_theme,
                            #[cfg(all(
                                unix,
                                not(any(
                                    target_os = "macos",
                                    target_os = "ios",
                                    target_os = "android"
                                ))
                            ))]
                            &mut popup_manager,
                        );
                        actions += 1;
                    }
                    event::Event::WindowEvent {
                        window_id: id,
                        event: event::WindowEvent::RedrawRequested,
                        ..
                    } => {
                        let Some(mut current_compositor) = compositor.as_mut() else {
                            continue;
                        };

                        let Some((id, mut window)) = window_manager.get_mut_alias(id) else {
                            continue;
                        };

                        let physical_size = window.state.physical_size();
                        let mut logical_size = window.state.logical_size();

                        if physical_size.width == 0 || physical_size.height == 0 {
                            continue;
                        }

                        // Window was resized between redraws
                        if window.surface_version != window.state.surface_version() {
                            #[cfg(feature = "hinting")]
                            window.renderer.hint(window.state.scale_factor());

                            let ui = user_interfaces.remove(&id).expect("Remove user interface");

                            let layout_span = debug::layout(id);
                            let _ = user_interfaces
                                .insert(id, ui.relayout(logical_size, &mut window.renderer));
                            layout_span.finish();

                            current_compositor.configure_surface(
                                &mut window.surface,
                                physical_size.width,
                                physical_size.height,
                            );

                            window.surface_version = window.state.surface_version();
                        }

                        let redraw_event =
                            core::Event::Window(window::Event::RedrawRequested(Instant::now()));

                        let cursor = window.state.cursor();

                        let mut interface =
                            user_interfaces.get_mut(&id).expect("Get user interface");

                        let interact_span = debug::interact(id);
                        let mut redraw_count = 0;

                        let state = loop {
                            let message_count = messages.len();
                            let (state, _) = interface.update(
                                slice::from_ref(&redraw_event),
                                cursor,
                                &mut window.renderer,
                                &mut messages,
                            );

                            if message_count == messages.len() && !state.has_layout_changed() {
                                break state;
                            }

                            if redraw_count >= 2 {
                                log::warn!(
                                    "More than 3 consecutive RedrawRequested events \
                                    produced layout invalidation"
                                );

                                break state;
                            }

                            redraw_count += 1;

                            if !messages.is_empty() {
                                let caches: FxHashMap<_, _> =
                                    ManuallyDrop::into_inner(user_interfaces)
                                        .into_iter()
                                        .map(|(id, interface)| (id, interface.into_cache()))
                                        .collect();

                                let actions = update(&mut program, &mut runtime, &mut messages);

                                user_interfaces = ManuallyDrop::new(build_user_interfaces(
                                    &program,
                                    &mut window_manager,
                                    caches,
                                ));

                                for action in actions {
                                    // Defer all window actions to avoid compositor
                                    // race conditions while redrawing
                                    if let Action::Window(_) = action {
                                        proxy.send_action(action);
                                        continue;
                                    }

                                    run_action(
                                        action,
                                        &program,
                                        &mut runtime,
                                        &mut compositor,
                                        &mut events,
                                        &mut messages,
                                        &mut clipboard,
                                        &mut control_sender,
                                        &mut user_interfaces,
                                        &mut window_manager,
                                        &mut ui_caches,
                                        &mut is_window_opening,
                                        &mut system_theme,
                                        #[cfg(all(
                                            unix,
                                            not(any(
                                                target_os = "macos",
                                                target_os = "ios",
                                                target_os = "android"
                                            ))
                                        ))]
                                        &mut popup_manager,
                                    );
                                }

                                for (window_id, window) in window_manager.iter_mut() {
                                    // We are already redrawing this window
                                    if window_id == id {
                                        continue;
                                    }

                                    window.raw.request_redraw();
                                }

                                let Some(next_compositor) = compositor.as_mut() else {
                                    continue 'next_event;
                                };

                                current_compositor = next_compositor;
                                window = window_manager.get_mut(id).unwrap();

                                // Window scale factor changed during a redraw request
                                if logical_size != window.state.logical_size() {
                                    logical_size = window.state.logical_size();

                                    log::debug!(
                                        "Window scale factor changed during a redraw request"
                                    );

                                    let ui =
                                        user_interfaces.remove(&id).expect("Remove user interface");

                                    let layout_span = debug::layout(id);
                                    let _ = user_interfaces.insert(
                                        id,
                                        ui.relayout(logical_size, &mut window.renderer),
                                    );
                                    layout_span.finish();
                                }

                                interface = user_interfaces.get_mut(&id).unwrap();
                            }
                        };
                        interact_span.finish();

                        let draw_span = debug::draw(id);
                        interface.draw(
                            &mut window.renderer,
                            window.state.theme(),
                            &renderer::Style {
                                text_color: window.state.text_color(),
                            },
                            cursor,
                        );
                        draw_span.finish();

                        if let user_interface::State::Updated {
                            redraw_request,
                            input_method,
                            mouse_interaction,
                            clipboard: mut clipboard_requests,
                            ..
                        } = state
                        {
                            window.request_redraw(redraw_request);
                            window.request_input_method(input_method);

                            // Only update the parent window's cursor if no popup
                            // currently owns cursor control. Otherwise the parent
                            // window resets the cursor to Default on every frame,
                            // fighting with the popup's Pointer cursor.
                            #[cfg(all(
                                unix,
                                not(any(
                                    target_os = "macos",
                                    target_os = "ios",
                                    target_os = "android"
                                ))
                            ))]
                            let popup_has_cursor = popup_manager
                                .iter()
                                .any(|(_, p)| p.parent_id == id && p.configured)
                                && popup_cursor_position.values().next().is_some();

                            #[cfg(not(all(
                                unix,
                                not(any(
                                    target_os = "macos",
                                    target_os = "ios",
                                    target_os = "android"
                                ))
                            )))]
                            let popup_has_cursor = false;

                            if !popup_has_cursor {
                                window.update_mouse(mouse_interaction);
                            }

                            resolve_dnd_icon_elements::<P, _>(
                                &mut clipboard_requests,
                                &mut *current_compositor,
                                window.state.viewport(),
                                window.state.theme(),
                                window.state.text_color(),
                            );

                            run_clipboard(
                                &mut proxy,
                                &mut clipboard,
                                clipboard_requests,
                                id,
                                Some(&window.raw),
                            );
                        } else if let user_interface::State::Outdated {
                            clipboard: mut clipboard_requests,
                        } = state
                        {
                            // Ensure DnD/clipboard requests are not lost when UI is outdated during redraw.
                            resolve_dnd_icon_elements::<P, _>(
                                &mut clipboard_requests,
                                &mut *current_compositor,
                                window.state.viewport(),
                                window.state.theme(),
                                window.state.text_color(),
                            );

                            run_clipboard(
                                &mut proxy,
                                &mut clipboard,
                                clipboard_requests,
                                id,
                                Some(&window.raw),
                            );
                        }

                        runtime.broadcast(subscription::Event::Interaction {
                            window: id,
                            event: redraw_event,
                            status: core::event::Status::Ignored,
                        });

                        window.draw_preedit();

                        let present_span = debug::present(id);
                        match current_compositor.present(
                            &mut window.renderer,
                            &mut window.surface,
                            window.state.viewport(),
                            window.state.background_color(),
                            || window.raw.pre_present_notify(),
                        ) {
                            Ok(()) => {
                                present_span.finish();

                                // Render child popups for this window
                                #[cfg(all(
                                    unix,
                                    not(any(
                                        target_os = "macos",
                                        target_os = "ios",
                                        target_os = "android"
                                    ))
                                ))]
                                {
                                    for (_popup_id, popup) in popup_manager.iter_mut() {
                                        // Only render popups that belong to this window
                                        if popup.parent_id != id || !popup.configured {
                                            continue;
                                        }

                                        if let (Some(surface), Some(renderer), Some(viewport)) = (
                                            popup.surface.as_mut(),
                                            popup.renderer.as_mut(),
                                            popup.viewport.as_ref(),
                                        ) {
                                            // Build or rebuild popup UI using cached state
                                            let popup_view = program.view(popup.iced_id);
                                            let logical_size = viewport.logical_size();

                                            let cache = ui_caches
                                                .remove(&popup.iced_id)
                                                .unwrap_or_default();

                                            let mut popup_ui = UserInterface::build(
                                                popup_view,
                                                Size::new(logical_size.width, logical_size.height),
                                                cache,
                                                renderer,
                                            );

                                            // Get cursor position for this popup
                                            let popup_cursor = popup_cursor_position
                                                .get(&popup.iced_id)
                                                .map(|pos| core::mouse::Cursor::Available(*pos))
                                                .unwrap_or(core::mouse::Cursor::Unavailable);

                                            // Send RedrawRequested through update() so widgets
                                            // (like button) can update their visual status
                                            // (hovered, pressed, etc.) based on cursor position.
                                            let redraw_event = core::Event::Window(
                                                window::Event::RedrawRequested(Instant::now()),
                                            );
                                            let (popup_state, _) = popup_ui.update(
                                                std::slice::from_ref(&redraw_event),
                                                popup_cursor,
                                                renderer,
                                                &mut messages,
                                            );

                                            // Draw the popup
                                            let _ = popup_ui.draw(
                                                renderer,
                                                window.state.theme(),
                                                &renderer::Style {
                                                    text_color: window.state.text_color(),
                                                },
                                                popup_cursor,
                                            );

                                            // Update cursor icon on parent window based on popup interaction
                                            if let user_interface::State::Updated {
                                                mouse_interaction,
                                                ..
                                            } = popup_state
                                            {
                                                window.update_mouse(mouse_interaction);
                                            }

                                            // Cache the UI for next frame
                                            let cache = popup_ui.into_cache();
                                            let _ = ui_caches.insert(popup.iced_id, cache);

                                            // Present the popup surface with transparent background
                                            // so rounded corners and shadows render correctly
                                            let _ = current_compositor.present(
                                                renderer,
                                                surface,
                                                viewport,
                                                crate::core::Color::TRANSPARENT,
                                                || {},
                                            );
                                        }
                                    }
                                }
                            }
                            Err(error) => match error {
                                compositor::SurfaceError::OutOfMemory => {
                                    // This is an unrecoverable error.
                                    panic!("{error:?}");
                                }
                                compositor::SurfaceError::Outdated
                                | compositor::SurfaceError::Lost => {
                                    present_span.finish();

                                    // Reconfigure surface and try redrawing
                                    let physical_size = window.state.physical_size();

                                    if error == compositor::SurfaceError::Lost {
                                        window.surface = current_compositor.create_surface(
                                            window.raw.clone(),
                                            physical_size.width,
                                            physical_size.height,
                                        );
                                    } else {
                                        current_compositor.configure_surface(
                                            &mut window.surface,
                                            physical_size.width,
                                            physical_size.height,
                                        );
                                    }

                                    window.raw.request_redraw();
                                }
                                _ => {
                                    present_span.finish();

                                    log::error!("Error {error:?} when presenting surface.");

                                    // Try rendering all windows again next frame.
                                    for (_id, window) in window_manager.iter_mut() {
                                        window.raw.request_redraw();
                                    }
                                }
                            },
                        }
                    }
                    event::Event::WindowEvent {
                        event: window_event,
                        window_id,
                    } => {
                        if !is_daemon
                            && matches!(window_event, winit::event::WindowEvent::Destroyed)
                            && !is_window_opening
                            && window_manager.is_empty()
                        {
                            control_sender
                                .start_send(Control::Exit)
                                .expect("Send control action");

                            continue;
                        }

                        let Some((id, window)) = window_manager.get_mut_alias(window_id) else {
                            continue;
                        };

                        match window_event {
                            winit::event::WindowEvent::Resized(_) => {
                                window.raw.request_redraw();
                            }
                            winit::event::WindowEvent::ThemeChanged(theme) => {
                                let mode = conversion::theme_mode(theme);

                                if mode != system_theme {
                                    system_theme = mode;

                                    runtime
                                        .broadcast(subscription::Event::SystemThemeChanged(mode));
                                }
                            }
                            _ => {}
                        }

                        if matches!(window_event, winit::event::WindowEvent::CloseRequested)
                            && window.exit_on_close_request
                        {
                            run_action(
                                Action::Window(runtime::window::Action::Close(id)),
                                &program,
                                &mut runtime,
                                &mut compositor,
                                &mut events,
                                &mut messages,
                                &mut clipboard,
                                &mut control_sender,
                                &mut user_interfaces,
                                &mut window_manager,
                                &mut ui_caches,
                                &mut is_window_opening,
                                &mut system_theme,
                                #[cfg(all(
                                    unix,
                                    not(any(
                                        target_os = "macos",
                                        target_os = "ios",
                                        target_os = "android"
                                    ))
                                ))]
                                &mut popup_manager,
                            );
                        } else {
                            window.state.update(&program, &window.raw, &window_event);

                            // If a mouse button is pressed on a window that has active popups,
                            // dismiss those popups (click-outside-to-close behavior).
                            #[cfg(all(
                                unix,
                                not(any(
                                    target_os = "macos",
                                    target_os = "ios",
                                    target_os = "android"
                                ))
                            ))]
                            if matches!(
                                window_event,
                                winit::event::WindowEvent::MouseInput {
                                    state: winit::event::ElementState::Pressed,
                                    ..
                                }
                            ) {
                                let child_popups: Vec<_> = popup_manager
                                    .iter()
                                    .filter(|(_, p)| p.parent_id == id && p.configured)
                                    .map(|(pid, p)| (*pid, p.iced_id, p.winit_popup_id))
                                    .collect();
                                for (popup_id, iced_id, winit_id) in child_popups {
                                    let _ = ui_caches.remove(&iced_id);
                                    let _ = user_interfaces.remove(&iced_id);
                                    let _ = popup_cursor_position.remove(&iced_id);
                                    let _ = popup_manager.remove(popup_id);
                                    events.push((
                                        iced_id,
                                        core::Event::Window(core::window::Event::Closed),
                                    ));
                                    if let Some(wid) = winit_id {
                                        let _ = control_sender.start_send(Control::DestroyPopup {
                                            winit_popup_id: wid,
                                        });
                                    }
                                }
                            }

                            if let Some(event) = conversion::window_event(
                                window_event,
                                window.state.scale_factor(),
                                window.state.modifiers(),
                            ) {
                                events.push((id, event));
                            }
                        }
                    }
                    event::Event::AboutToWait => {
                        if actions > 0 {
                            proxy.free_slots(actions);
                            actions = 0;
                        }

                        if events.is_empty() && messages.is_empty() && window_manager.is_idle() {
                            continue;
                        }

                        let mut uis_stale = false;

                        for (id, window) in window_manager.iter_mut() {
                            let interact_span = debug::interact(id);
                            let mut window_events = vec![];

                            events.retain(|(window_id, event)| {
                                if *window_id == id {
                                    window_events.push(event.clone());
                                    false
                                } else {
                                    true
                                }
                            });

                            if window_events.is_empty() {
                                continue;
                            }

                            let (ui_state, statuses) = user_interfaces
                                .get_mut(&id)
                                .expect("Get user interface")
                                .update(
                                    &window_events,
                                    window.state.cursor(),
                                    &mut window.renderer,
                                    &mut messages,
                                );

                            #[cfg(feature = "unconditional-rendering")]
                            window.request_redraw(window::RedrawRequest::NextFrame);

                            match &ui_state {
                                user_interface::State::Updated { clipboard: cb, .. }
                                | user_interface::State::Outdated { clipboard: cb } => {
                                    if !cb.dnd_requests.is_empty() {
                                        tracing::trace!(
                                            "[WINIT] ui_state has {} dnd_requests",
                                            cb.dnd_requests.len()
                                        );
                                    }
                                }
                            }

                            match ui_state {
                                user_interface::State::Updated {
                                    redraw_request: _redraw_request,
                                    mouse_interaction,
                                    clipboard: mut clipboard_requests,
                                    ..
                                } => {
                                    window.update_mouse(mouse_interaction);

                                    #[cfg(not(feature = "unconditional-rendering"))]
                                    window.request_redraw(_redraw_request);

                                    if let Some(ref mut comp) = compositor {
                                        resolve_dnd_icon_elements::<P, _>(
                                            &mut clipboard_requests,
                                            comp,
                                            window.state.viewport(),
                                            window.state.theme(),
                                            window.state.text_color(),
                                        );
                                    }

                                    run_clipboard(
                                        &mut proxy,
                                        &mut clipboard,
                                        clipboard_requests,
                                        id,
                                        Some(&window.raw),
                                    );
                                }
                                user_interface::State::Outdated {
                                    clipboard: mut clipboard_requests,
                                } => {
                                    if let Some(ref mut comp) = compositor {
                                        resolve_dnd_icon_elements::<P, _>(
                                            &mut clipboard_requests,
                                            comp,
                                            window.state.viewport(),
                                            window.state.theme(),
                                            window.state.text_color(),
                                        );
                                    }

                                    run_clipboard(
                                        &mut proxy,
                                        &mut clipboard,
                                        clipboard_requests,
                                        id,
                                        Some(&window.raw),
                                    );
                                    uis_stale = true;
                                }
                            }

                            for (event, status) in
                                window_events.into_iter().zip(statuses.into_iter())
                            {
                                runtime.broadcast(subscription::Event::Interaction {
                                    window: id,
                                    event,
                                    status,
                                });
                            }

                            interact_span.finish();
                        }

                        // Process popup events
                        #[cfg(all(
                            unix,
                            not(any(
                                target_os = "macos",
                                target_os = "ios",
                                target_os = "android"
                            ))
                        ))]
                        {
                            for (_popup_id, popup) in popup_manager.iter_mut() {
                                if !popup.configured {
                                    continue;
                                }

                                let iced_id = popup.iced_id;
                                let mut popup_events = vec![];

                                events.retain(|(window_id, event)| {
                                    if *window_id == iced_id {
                                        popup_events.push(event.clone());
                                        false
                                    } else {
                                        true
                                    }
                                });

                                if popup_events.is_empty() {
                                    continue;
                                }

                                if let (Some(renderer), Some(viewport)) =
                                    (popup.renderer.as_mut(), popup.viewport.as_ref())
                                {
                                    let popup_view = program.view(iced_id);
                                    let logical_size = viewport.logical_size();

                                    let cache = ui_caches.remove(&iced_id).unwrap_or_default();

                                    let mut popup_ui = UserInterface::build(
                                        popup_view,
                                        Size::new(logical_size.width, logical_size.height),
                                        cache,
                                        renderer,
                                    );

                                    let popup_cursor = popup_cursor_position
                                        .get(&iced_id)
                                        .map(|pos| core::mouse::Cursor::Available(*pos))
                                        .unwrap_or(core::mouse::Cursor::Unavailable);

                                    let (_ui_state, _statuses) = popup_ui.update(
                                        &popup_events,
                                        popup_cursor,
                                        renderer,
                                        &mut messages,
                                    );

                                    // Cache the UI for rendering
                                    let cache = popup_ui.into_cache();
                                    let _ = ui_caches.insert(iced_id, cache);

                                    // Request parent window redraw to trigger popup re-render
                                    if let Some((_id, parent_window)) = window_manager
                                        .iter_mut()
                                        .find(|(wid, _)| *wid == popup.parent_id)
                                    {
                                        parent_window.raw.request_redraw();
                                    }
                                }
                            }
                        }

                        for (id, event) in events.drain(..) {
                            runtime.broadcast(subscription::Event::Interaction {
                                window: id,
                                event,
                                status: core::event::Status::Ignored,
                            });
                        }

                        if !messages.is_empty() || uis_stale {
                            let cached_interfaces: FxHashMap<_, _> =
                                ManuallyDrop::into_inner(user_interfaces)
                                    .into_iter()
                                    .map(|(id, ui)| (id, ui.into_cache()))
                                    .collect();

                            let actions = update(&mut program, &mut runtime, &mut messages);

                            user_interfaces = ManuallyDrop::new(build_user_interfaces(
                                &program,
                                &mut window_manager,
                                cached_interfaces,
                            ));

                            for action in actions {
                                run_action(
                                    action,
                                    &program,
                                    &mut runtime,
                                    &mut compositor,
                                    &mut events,
                                    &mut messages,
                                    &mut clipboard,
                                    &mut control_sender,
                                    &mut user_interfaces,
                                    &mut window_manager,
                                    &mut ui_caches,
                                    &mut is_window_opening,
                                    &mut system_theme,
                                    #[cfg(all(
                                        unix,
                                        not(any(
                                            target_os = "macos",
                                            target_os = "ios",
                                            target_os = "android"
                                        ))
                                    ))]
                                    &mut popup_manager,
                                );
                            }

                            for (_id, window) in window_manager.iter_mut() {
                                window.raw.request_redraw();
                            }
                        }

                        if let Some(redraw_at) = window_manager.redraw_at() {
                            let _ = control_sender
                                .start_send(Control::ChangeFlow(ControlFlow::WaitUntil(redraw_at)));
                        } else {
                            let _ =
                                control_sender.start_send(Control::ChangeFlow(ControlFlow::Wait));
                        }
                    }
                    _ => {}
                }
            }
            Event::Exit => break,
        }
    }

    let _ = ManuallyDrop::into_inner(user_interfaces);
}

/// Builds a window's [`UserInterface`] for the [`Program`].
fn build_user_interface<'a, P: Program>(
    program: &'a program::Instance<P>,
    cache: user_interface::Cache,
    renderer: &mut P::Renderer,
    size: Size,
    id: window::Id,
) -> UserInterface<'a, P::Message, P::Theme, P::Renderer>
where
    P::Theme: theme::Base,
{
    let view_span = debug::view(id);
    let view = program.view(id);
    view_span.finish();

    let layout_span = debug::layout(id);
    let user_interface = UserInterface::build(view, size, cache, renderer);
    layout_span.finish();

    user_interface
}

fn update<P: Program, E: Executor>(
    program: &mut program::Instance<P>,
    runtime: &mut Runtime<E, Proxy<P::Message>, Action<P::Message>>,
    messages: &mut Vec<P::Message>,
) -> Vec<Action<P::Message>>
where
    P::Theme: theme::Base,
{
    use futures::futures;

    let mut actions = Vec::new();
    let mut outputs = Vec::new();

    while !messages.is_empty() {
        for message in messages.drain(..) {
            let task = runtime.enter(|| program.update(message));

            if let Some(mut stream) = runtime::task::into_stream(task) {
                let waker = futures::task::noop_waker_ref();
                let mut context = futures::task::Context::from_waker(waker);

                // Run immediately available actions synchronously (e.g. widget operations)
                loop {
                    match runtime.enter(|| stream.poll_next_unpin(&mut context)) {
                        futures::task::Poll::Ready(Some(Action::Output(output))) => {
                            outputs.push(output);
                        }
                        futures::task::Poll::Ready(Some(action)) => {
                            actions.push(action);
                        }
                        futures::task::Poll::Ready(None) => {
                            break;
                        }
                        futures::task::Poll::Pending => {
                            runtime.run(stream);
                            break;
                        }
                    }
                }
            }
        }

        messages.append(&mut outputs);
    }

    let subscription = runtime.enter(|| program.subscription());
    let recipes = subscription::into_recipes(subscription.map(Action::Output));

    runtime.track(recipes);

    actions
}

fn run_action<'a, P, C>(
    action: Action<P::Message>,
    program: &'a program::Instance<P>,
    runtime: &mut Runtime<P::Executor, Proxy<P::Message>, Action<P::Message>>,
    compositor: &mut Option<C>,
    events: &mut Vec<(window::Id, core::Event)>,
    messages: &mut Vec<P::Message>,
    clipboard: &mut Clipboard,
    control_sender: &mut mpsc::UnboundedSender<Control>,
    interfaces: &mut FxHashMap<window::Id, UserInterface<'a, P::Message, P::Theme, P::Renderer>>,
    window_manager: &mut WindowManager<P, C>,
    ui_caches: &mut FxHashMap<window::Id, user_interface::Cache>,
    is_window_opening: &mut bool,
    system_theme: &mut theme::Mode,
    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ))]
    popup_manager: &mut popup::PopupManager<P, C>,
) where
    P: Program,
    C: Compositor<Renderer = P::Renderer> + 'static,
    P::Theme: theme::Base,
{
    use crate::core::Renderer as _;
    use crate::runtime::clipboard;
    use crate::runtime::user_interface::{self, UserInterface};
    use crate::runtime::window;

    match action {
        Action::Output(message) => {
            messages.push(message);
        }
        Action::Clipboard(action) => match action {
            clipboard::Action::Read { kind, channel } => {
                clipboard.read(kind, move |result| {
                    let _ = channel.send(result);
                });
            }
            clipboard::Action::Write { content, channel } => {
                clipboard.write(content, move |result| {
                    let _ = channel.send(result);
                });
            }
        },
        Action::Window(action) => match action {
            window::Action::Open(id, settings, channel) => {
                let monitor = window_manager.last_monitor();

                control_sender
                    .start_send(Control::CreateWindow {
                        id,
                        settings,
                        title: program.title(id),
                        scale_factor: program.scale_factor(id),
                        monitor,
                        on_open: channel,
                    })
                    .expect("Send control action");

                *is_window_opening = true;
            }
            window::Action::Close(id) => {
                let _ = ui_caches.remove(&id);
                let _ = interfaces.remove(&id);

                if window_manager.remove(id).is_some() {
                    events.push((id, core::Event::Window(core::window::Event::Closed)));
                }

                if window_manager.is_empty() {
                    *compositor = None;
                }
            }
            window::Action::GetOldest(channel) => {
                let id = window_manager.iter_mut().next().map(|(id, _window)| id);

                let _ = channel.send(id);
            }
            window::Action::GetLatest(channel) => {
                let id = window_manager.iter_mut().last().map(|(id, _window)| id);

                let _ = channel.send(id);
            }
            window::Action::Drag(id) => {
                if let Some(window) = window_manager.get_mut(id) {
                    let _ = window.raw.drag_window();
                }
            }
            window::Action::DragResize(id, direction) => {
                if let Some(window) = window_manager.get_mut(id) {
                    let _ = window
                        .raw
                        .drag_resize_window(conversion::resize_direction(direction));
                }
            }
            window::Action::Resize(id, size) => {
                if let Some(window) = window_manager.get_mut(id) {
                    let _ = window.raw.request_inner_size(
                        winit::dpi::LogicalSize {
                            width: size.width,
                            height: size.height,
                        }
                        .to_physical::<f32>(f64::from(window.state.scale_factor())),
                    );
                }
            }
            window::Action::AnimatedResize(id, width, height, duration_ms) => {
                if let Some(window) = window_manager.get_mut(id) {
                    #[cfg(all(
                        feature = "wayland",
                        any(
                            target_os = "linux",
                            target_os = "dragonfly",
                            target_os = "freebsd",
                            target_os = "netbsd",
                            target_os = "openbsd",
                        )
                    ))]
                    {
                        use winit::platform::wayland::WindowExtWayland;
                        let _ = window.raw.request_animated_resize(
                            width as i32,
                            height as i32,
                            duration_ms,
                        );
                    }
                    #[cfg(not(all(
                        feature = "wayland",
                        any(
                            target_os = "linux",
                            target_os = "dragonfly",
                            target_os = "freebsd",
                            target_os = "netbsd",
                            target_os = "openbsd",
                        )
                    )))]
                    {
                        // Fall back to instant resize on non-Wayland platforms
                        let _ = window.raw.request_inner_size(
                            winit::dpi::LogicalSize {
                                width: width as f32,
                                height: height as f32,
                            }
                            .to_physical::<f32>(f64::from(window.state.scale_factor())),
                        );
                    }
                }
            }
            window::Action::AnimatedResizeWithPosition(id, x, y, width, height, duration_ms) => {
                if let Some(window) = window_manager.get_mut(id) {
                    #[cfg(all(
                        feature = "wayland",
                        any(
                            target_os = "linux",
                            target_os = "dragonfly",
                            target_os = "freebsd",
                            target_os = "netbsd",
                            target_os = "openbsd",
                        )
                    ))]
                    {
                        use winit::platform::wayland::WindowExtWayland;
                        let _ = window.raw.request_animated_resize_with_position(
                            x,
                            y,
                            width as i32,
                            height as i32,
                            duration_ms,
                        );
                    }
                    #[cfg(not(all(
                        feature = "wayland",
                        any(
                            target_os = "linux",
                            target_os = "dragonfly",
                            target_os = "freebsd",
                            target_os = "netbsd",
                            target_os = "openbsd",
                        )
                    )))]
                    {
                        // Fall back to instant resize on non-Wayland platforms (position ignored)
                        let _ = window.raw.request_inner_size(
                            winit::dpi::LogicalSize {
                                width: width as f32,
                                height: height as f32,
                            }
                            .to_physical::<f32>(f64::from(window.state.scale_factor())),
                        );
                    }
                }
            }
            window::Action::SetMinSize(id, size) => {
                if let Some(window) = window_manager.get_mut(id) {
                    window.raw.set_min_inner_size(size.map(|size| {
                        winit::dpi::LogicalSize {
                            width: size.width,
                            height: size.height,
                        }
                        .to_physical::<f32>(f64::from(window.state.scale_factor()))
                    }));
                }
            }
            window::Action::SetMaxSize(id, size) => {
                if let Some(window) = window_manager.get_mut(id) {
                    window.raw.set_max_inner_size(size.map(|size| {
                        winit::dpi::LogicalSize {
                            width: size.width,
                            height: size.height,
                        }
                        .to_physical::<f32>(f64::from(window.state.scale_factor()))
                    }));
                }
            }
            window::Action::SetResizeIncrements(id, increments) => {
                if let Some(window) = window_manager.get_mut(id) {
                    window.raw.set_resize_increments(increments.map(|size| {
                        winit::dpi::LogicalSize {
                            width: size.width,
                            height: size.height,
                        }
                        .to_physical::<f32>(f64::from(window.state.scale_factor()))
                    }));
                }
            }
            window::Action::SetResizable(id, resizable) => {
                if let Some(window) = window_manager.get_mut(id) {
                    window.raw.set_resizable(resizable);
                }
            }
            window::Action::GetSize(id, channel) => {
                if let Some(window) = window_manager.get_mut(id) {
                    let size = window.logical_size();
                    let _ = channel.send(Size::new(size.width, size.height));
                }
            }
            window::Action::GetMaximized(id, channel) => {
                if let Some(window) = window_manager.get_mut(id) {
                    let _ = channel.send(window.raw.is_maximized());
                }
            }
            window::Action::Maximize(id, maximized) => {
                if let Some(window) = window_manager.get_mut(id) {
                    window.raw.set_maximized(maximized);
                }
            }
            window::Action::GetMinimized(id, channel) => {
                if let Some(window) = window_manager.get_mut(id) {
                    let _ = channel.send(window.raw.is_minimized());
                }
            }
            window::Action::Minimize(id, minimized) => {
                if let Some(window) = window_manager.get_mut(id) {
                    window.raw.set_minimized(minimized);
                }
            }
            window::Action::GetPosition(id, channel) => {
                if let Some(window) = window_manager.get(id) {
                    let position = window
                        .raw
                        .outer_position()
                        .map(|position| {
                            let position = position.to_logical::<f32>(window.raw.scale_factor());

                            Point::new(position.x, position.y)
                        })
                        .ok();

                    let _ = channel.send(position);
                }
            }
            window::Action::GetScaleFactor(id, channel) => {
                if let Some(window) = window_manager.get_mut(id) {
                    let scale_factor = window.raw.scale_factor();

                    let _ = channel.send(scale_factor as f32);
                }
            }
            window::Action::Move(id, position) => {
                if let Some(window) = window_manager.get_mut(id) {
                    window.raw.set_outer_position(winit::dpi::LogicalPosition {
                        x: position.x,
                        y: position.y,
                    });
                }
            }
            window::Action::SetMode(id, mode) => {
                if let Some(window) = window_manager.get_mut(id) {
                    window.raw.set_visible(conversion::visible(mode));
                    window
                        .raw
                        .set_fullscreen(conversion::fullscreen(window.raw.current_monitor(), mode));
                }
            }
            window::Action::SetIcon(id, icon) => {
                if let Some(window) = window_manager.get_mut(id) {
                    window.raw.set_window_icon(conversion::icon(icon));
                }
            }
            window::Action::GetMode(id, channel) => {
                if let Some(window) = window_manager.get_mut(id) {
                    let mode = if window.raw.is_visible().unwrap_or(true) {
                        conversion::mode(window.raw.fullscreen())
                    } else {
                        core::window::Mode::Hidden
                    };

                    let _ = channel.send(mode);
                }
            }
            window::Action::ToggleMaximize(id) => {
                if let Some(window) = window_manager.get_mut(id) {
                    window.raw.set_maximized(!window.raw.is_maximized());
                }
            }
            window::Action::ToggleDecorations(id) => {
                if let Some(window) = window_manager.get_mut(id) {
                    window.raw.set_decorations(!window.raw.is_decorated());
                }
            }
            window::Action::RequestUserAttention(id, attention_type) => {
                if let Some(window) = window_manager.get_mut(id) {
                    window
                        .raw
                        .request_user_attention(attention_type.map(conversion::user_attention));
                }
            }
            window::Action::GainFocus(id) => {
                if let Some(window) = window_manager.get_mut(id) {
                    window.raw.focus_window();
                }
            }
            window::Action::SetLevel(id, level) => {
                if let Some(window) = window_manager.get_mut(id) {
                    window.raw.set_window_level(conversion::window_level(level));
                }
            }
            window::Action::ShowSystemMenu(id) => {
                if let Some(window) = window_manager.get_mut(id)
                    && let mouse::Cursor::Available(point) = window.state.cursor()
                {
                    window.raw.show_window_menu(winit::dpi::LogicalPosition {
                        x: point.x,
                        y: point.y,
                    });
                }
            }
            window::Action::GetRawId(id, channel) => {
                if let Some(window) = window_manager.get_mut(id) {
                    let _ = channel.send(window.raw.id().into());
                }
            }
            window::Action::Run(id, f) => {
                if let Some(window) = window_manager.get_mut(id) {
                    f(window);
                }
            }
            window::Action::Screenshot(id, channel) => {
                if let Some(window) = window_manager.get_mut(id)
                    && let Some(compositor) = compositor
                {
                    let bytes = compositor.screenshot(
                        &mut window.renderer,
                        window.state.viewport(),
                        window.state.background_color(),
                    );

                    let _ = channel.send(core::window::Screenshot::new(
                        bytes,
                        window.state.physical_size(),
                        window.state.scale_factor(),
                    ));
                }
            }
            window::Action::EnableMousePassthrough(id) => {
                if let Some(window) = window_manager.get_mut(id) {
                    let _ = window.raw.set_cursor_hittest(false);
                }
            }
            window::Action::DisableMousePassthrough(id) => {
                if let Some(window) = window_manager.get_mut(id) {
                    let _ = window.raw.set_cursor_hittest(true);
                }
            }
            window::Action::GetMonitorSize(id, channel) => {
                if let Some(window) = window_manager.get(id) {
                    let size = window.raw.current_monitor().map(|monitor| {
                        let scale = window.state.scale_factor();
                        let size = monitor.size().to_logical(f64::from(scale));

                        Size::new(size.width, size.height)
                    });

                    let _ = channel.send(size);
                }
            }
            window::Action::SetAllowAutomaticTabbing(enabled) => {
                control_sender
                    .start_send(Control::SetAutomaticWindowTabbing(enabled))
                    .expect("Send control action");
            }
            window::Action::RedrawAll => {
                for (_id, window) in window_manager.iter_mut() {
                    window.raw.request_redraw();
                }
            }
            window::Action::RelayoutAll => {
                for (id, window) in window_manager.iter_mut() {
                    if let Some(ui) = interfaces.remove(&id) {
                        let _ = interfaces.insert(
                            id,
                            ui.relayout(window.state.logical_size(), &mut window.renderer),
                        );
                    }

                    window.raw.request_redraw();
                }
            }
            window::Action::EmbedToplevelByPid(
                id,
                pid,
                app_id,
                x,
                y,
                width,
                height,
                interactive,
                channel,
            ) => {
                #[cfg(all(
                    feature = "wayland",
                    any(
                        target_os = "linux",
                        target_os = "dragonfly",
                        target_os = "freebsd",
                        target_os = "netbsd",
                        target_os = "openbsd",
                    )
                ))]
                {
                    use winit::platform::wayland::WindowExtWayland;
                    if let Some(window) = window_manager.get_mut(id) {
                        let result = window.raw.embed_toplevel_by_pid(
                            pid,
                            &app_id,
                            x,
                            y,
                            width,
                            height,
                            interactive,
                        );
                        let _ = channel.send(result);
                    } else {
                        let _ = channel.send(None);
                    }
                }
                #[cfg(not(all(
                    feature = "wayland",
                    any(
                        target_os = "linux",
                        target_os = "dragonfly",
                        target_os = "freebsd",
                        target_os = "netbsd",
                        target_os = "openbsd",
                    )
                )))]
                {
                    // Embedding not supported on non-Wayland platforms
                    let _ = channel.send(None);
                }
            }
            window::Action::SetEmbedGeometry(id, embed_id, x, y, width, height) => {
                #[cfg(all(
                    feature = "wayland",
                    any(
                        target_os = "linux",
                        target_os = "dragonfly",
                        target_os = "freebsd",
                        target_os = "netbsd",
                        target_os = "openbsd",
                    )
                ))]
                {
                    use winit::platform::wayland::WindowExtWayland;
                    if let Some(window) = window_manager.get_mut(id) {
                        let _ = window.raw.set_embed_geometry(embed_id, x, y, width, height);
                    }
                }
            }
            window::Action::SetEmbedAnchor(
                id,
                embed_id,
                anchor,
                margin_top,
                margin_right,
                margin_bottom,
                margin_left,
                width,
                height,
            ) => {
                #[cfg(all(
                    feature = "wayland",
                    any(
                        target_os = "linux",
                        target_os = "dragonfly",
                        target_os = "freebsd",
                        target_os = "netbsd",
                        target_os = "openbsd",
                    )
                ))]
                {
                    use winit::platform::wayland::WindowExtWayland;
                    if let Some(window) = window_manager.get_mut(id) {
                        let _ = window.raw.set_embed_anchor(
                            embed_id,
                            anchor,
                            margin_top,
                            margin_right,
                            margin_bottom,
                            margin_left,
                            width,
                            height,
                        );
                    }
                }
            }
            window::Action::SetEmbedCornerRadius(
                id,
                embed_id,
                top_left,
                top_right,
                bottom_right,
                bottom_left,
            ) => {
                #[cfg(all(
                    feature = "wayland",
                    any(
                        target_os = "linux",
                        target_os = "dragonfly",
                        target_os = "freebsd",
                        target_os = "netbsd",
                        target_os = "openbsd",
                    )
                ))]
                {
                    use winit::platform::wayland::WindowExtWayland;
                    if let Some(window) = window_manager.get_mut(id) {
                        let _ = window.raw.set_embed_corner_radius(
                            embed_id,
                            top_left,
                            top_right,
                            bottom_right,
                            bottom_left,
                        );
                    }
                }
            }
            window::Action::SetEmbedInteractive(id, embed_id, interactive) => {
                #[cfg(all(
                    feature = "wayland",
                    any(
                        target_os = "linux",
                        target_os = "dragonfly",
                        target_os = "freebsd",
                        target_os = "netbsd",
                        target_os = "openbsd",
                    )
                ))]
                {
                    use winit::platform::wayland::WindowExtWayland;
                    if let Some(window) = window_manager.get_mut(id) {
                        let _ = window.raw.set_embed_interactive(embed_id, interactive);
                    }
                }
            }
            window::Action::RemoveEmbed(id, embed_id) => {
                #[cfg(all(
                    feature = "wayland",
                    any(
                        target_os = "linux",
                        target_os = "dragonfly",
                        target_os = "freebsd",
                        target_os = "netbsd",
                        target_os = "openbsd",
                    )
                ))]
                {
                    use winit::platform::wayland::WindowExtWayland;
                    if let Some(window) = window_manager.get_mut(id) {
                        let _ = window.raw.remove_embed(embed_id);
                    }
                }
            }
            window::Action::SetExclusiveMode(id, exclusive) => {
                #[cfg(all(
                    feature = "wayland",
                    any(
                        target_os = "linux",
                        target_os = "dragonfly",
                        target_os = "freebsd",
                        target_os = "netbsd",
                        target_os = "openbsd",
                    )
                ))]
                {
                    use winit::platform::wayland::WindowExtWayland;
                    if let Some(window) = window_manager.get_mut(id) {
                        let _ = window.raw.set_exclusive_mode(exclusive);
                    }
                }
            }
            window::Action::SetCornerRadius(id, top_left, top_right, bottom_right, bottom_left) => {
                #[cfg(all(
                    feature = "wayland",
                    any(
                        target_os = "linux",
                        target_os = "dragonfly",
                        target_os = "freebsd",
                        target_os = "netbsd",
                        target_os = "openbsd",
                    )
                ))]
                {
                    use winit::platform::wayland::WindowExtWayland;
                    if let Some(window) = window_manager.get_mut(id) {
                        let _ = window.raw.set_corner_radius(
                            top_left,
                            top_right,
                            bottom_right,
                            bottom_left,
                        );
                    }
                }
            }
            window::Action::SetBackdropColor(id, r, g, b, a) => {
                #[cfg(all(
                    feature = "wayland",
                    any(
                        target_os = "linux",
                        target_os = "dragonfly",
                        target_os = "freebsd",
                        target_os = "netbsd",
                        target_os = "openbsd",
                    )
                ))]
                {
                    use winit::platform::wayland::WindowExtWayland;
                    if let Some(window) = window_manager.get_mut(id) {
                        let _ = window.raw.set_backdrop_color(r, g, b, a);
                    }
                }
            }
            window::Action::RegisterVoiceMode(id, is_default) => {
                #[cfg(all(
                    feature = "wayland",
                    any(
                        target_os = "linux",
                        target_os = "dragonfly",
                        target_os = "freebsd",
                        target_os = "netbsd",
                        target_os = "openbsd",
                    )
                ))]
                {
                    use winit::platform::wayland::WindowExtWayland;
                    if let Some(window) = window_manager.get_mut(id) {
                        let _ = window.raw.register_voice_mode(is_default);
                    }
                }
            }
            window::Action::UnregisterVoiceMode(id) => {
                #[cfg(all(
                    feature = "wayland",
                    any(
                        target_os = "linux",
                        target_os = "dragonfly",
                        target_os = "freebsd",
                        target_os = "netbsd",
                        target_os = "openbsd",
                    )
                ))]
                {
                    use winit::platform::wayland::WindowExtWayland;
                    if let Some(window) = window_manager.get_mut(id) {
                        let _ = window.raw.unregister_voice_mode();
                    }
                }
            }
            window::Action::SetVoiceAudioLevel(level) => {
                #[cfg(all(
                    feature = "wayland",
                    any(
                        target_os = "linux",
                        target_os = "dragonfly",
                        target_os = "freebsd",
                        target_os = "netbsd",
                        target_os = "openbsd",
                    )
                ))]
                {
                    use winit::platform::wayland::WindowExtWayland;
                    // Send audio level to first registered window (audio level is global)
                    for (_id, window) in window_manager.iter_mut() {
                        let _ = window.raw.set_voice_audio_level(level);
                        break;
                    }
                }
            }
            window::Action::VoiceAckStop(id, serial, freeze) => {
                #[cfg(all(
                    feature = "wayland",
                    any(
                        target_os = "linux",
                        target_os = "dragonfly",
                        target_os = "freebsd",
                        target_os = "netbsd",
                        target_os = "openbsd",
                    )
                ))]
                {
                    use winit::platform::wayland::WindowExtWayland;
                    if let Some(window) = window_manager.get_mut(id) {
                        let _ = window.raw.voice_ack_stop(serial, freeze);
                    }
                }
            }
            window::Action::VoiceDismiss(id) => {
                #[cfg(all(
                    feature = "wayland",
                    any(
                        target_os = "linux",
                        target_os = "dragonfly",
                        target_os = "freebsd",
                        target_os = "netbsd",
                        target_os = "openbsd",
                    )
                ))]
                {
                    use winit::platform::wayland::WindowExtWayland;
                    if let Some(window) = window_manager.get_mut(id) {
                        let _ = window.raw.voice_dismiss();
                    }
                }
            }
        },
        Action::System(action) => match action {
            system::Action::GetInformation(_channel) => {
                #[cfg(feature = "sysinfo")]
                {
                    if let Some(compositor) = compositor {
                        let graphics_info = compositor.information();

                        let _ = std::thread::spawn(move || {
                            let information = system_information(graphics_info);

                            let _ = _channel.send(information);
                        });
                    }
                }
            }
            system::Action::GetTheme(channel) => {
                let _ = channel.send(*system_theme);
            }
            system::Action::NotifyTheme(mode) => {
                if mode != *system_theme {
                    *system_theme = mode;

                    runtime.broadcast(subscription::Event::SystemThemeChanged(mode));
                }

                let Some(theme) = conversion::window_theme(mode) else {
                    return;
                };

                for (_id, window) in window_manager.iter_mut() {
                    window.state.update(
                        program,
                        &window.raw,
                        &winit::event::WindowEvent::ThemeChanged(theme),
                    );
                }
            }
        },
        Action::Widget(operation) => {
            let mut current_operation = Some(operation);

            while let Some(mut operation) = current_operation.take() {
                for (id, ui) in interfaces.iter_mut() {
                    if let Some(window) = window_manager.get_mut(*id) {
                        ui.operate(&window.renderer, operation.as_mut());
                    }
                }

                match operation.finish() {
                    operation::Outcome::None => {}
                    operation::Outcome::Some(()) => {}
                    operation::Outcome::Chain(next) => {
                        current_operation = Some(next);
                    }
                }
            }

            // Redraw all windows
            for (_, window) in window_manager.iter_mut() {
                window.raw.request_redraw();
            }
        }
        Action::Image(action) => match action {
            image::Action::Allocate(handle, sender) => {
                // TODO: Shared image cache in compositor
                if let Some((_id, window)) = window_manager.iter_mut().next() {
                    window.renderer.allocate_image(&handle, move |allocation| {
                        let _ = sender.send(allocation);
                    });
                }
            }
        },
        Action::Event { window, event } => {
            events.push((window, event));
        }
        Action::LoadFont { bytes, channel } => {
            if let Some(compositor) = compositor {
                // TODO: Error handling (?)
                compositor.load_font(bytes.clone());

                let _ = channel.send(Ok(()));
            }
        }
        Action::Tick => {
            for (_id, window) in window_manager.iter_mut() {
                window.renderer.tick();
            }
        }
        Action::Reload => {
            for (id, window) in window_manager.iter_mut() {
                let Some(ui) = interfaces.remove(&id) else {
                    continue;
                };

                let cache = ui.into_cache();
                let size = window.logical_size();

                let _ = interfaces.insert(
                    id,
                    build_user_interface(program, cache, &mut window.renderer, size, id),
                );

                window.raw.request_redraw();
            }
        }
        Action::Exit => {
            control_sender
                .start_send(Control::Exit)
                .expect("Send control action");
        }
        #[cfg(all(
            unix,
            not(any(target_os = "macos", target_os = "ios", target_os = "android"))
        ))]
        Action::PlatformSpecific(action) => {
            use crate::runtime::platform_specific;

            match action {
                platform_specific::Action::Wayland(wayland_action) => {
                    use crate::runtime::platform_specific::wayland::{
                        Action as WaylandAction, popup,
                    };

                    match wayland_action {
                        WaylandAction::Popup(popup_action) => match popup_action {
                            popup::Action::Show { settings } => {
                                // Get the parent window's winit ID from the window manager
                                let parent_winit_id = match window_manager.get(settings.parent) {
                                    Some(parent_window) => parent_window.raw.id(),
                                    None => {
                                        tracing::warn!(
                                            "Parent window {:?} not found for popup",
                                            settings.parent
                                        );
                                        return;
                                    }
                                };

                                // Destroy any existing popups for this parent
                                let existing: Vec<_> = popup_manager
                                    .iter()
                                    .filter(|(_, p)| p.parent_id == settings.parent)
                                    .map(|(id, p)| (*id, p.iced_id, p.winit_popup_id))
                                    .collect();
                                for (popup_id, iced_id, winit_id) in existing {
                                    let _ = ui_caches.remove(&iced_id);
                                    let _ = interfaces.remove(&iced_id);
                                    let _ = popup_manager.remove(popup_id);
                                    events.push((
                                        iced_id,
                                        core::Event::Window(core::window::Event::Closed),
                                    ));
                                    if let Some(wid) = winit_id {
                                        let _ = control_sender.start_send(Control::DestroyPopup {
                                            winit_popup_id: wid,
                                        });
                                    }
                                }

                                // Determine popup size: use explicit size or auto-measure content
                                let anchor_rect = settings.positioner.anchor_rect;
                                let shadow_pad = settings.positioner.shadow_padding;
                                let (size, window_geometry) =
                                    if let Some(size) = settings.positioner.size {
                                        (size, settings.positioner.window_geometry)
                                    } else if let Some(comp) = compositor.as_mut() {
                                        // Auto-size: measure the popup content to determine size
                                        let mut measure_renderer = comp.create_renderer();
                                        let popup_view = program.view(settings.id);
                                        let max_size = settings.positioner.size_limits.max();
                                        let ui = UserInterface::build(
                                            popup_view,
                                            max_size,
                                            user_interface::Cache::default(),
                                            &mut measure_renderer,
                                        );
                                        let content = ui.content_size();
                                        let w = content.width.ceil() as u32;
                                        let h = content.height.ceil() as u32;
                                        // Compute window_geometry from shadow_padding
                                        let wg = if shadow_pad > 0 {
                                            let sp = shadow_pad as i32;
                                            let inner_w = w as i32 - 2 * sp;
                                            let inner_h = h as i32 - 2 * sp;
                                            if inner_w > 0 && inner_h > 0 {
                                                Some((sp, sp, inner_w, inner_h))
                                            } else {
                                                None
                                            }
                                        } else {
                                            settings.positioner.window_geometry
                                        };
                                        ((w, h), wg)
                                    } else {
                                        ((200, 200), settings.positioner.window_geometry)
                                    };

                                control_sender
                                    .start_send(Control::CreatePopup {
                                        id: settings.id,
                                        parent_iced_id: settings.parent,
                                        parent_winit_id,
                                        size,
                                        anchor_rect: (
                                            anchor_rect.x,
                                            anchor_rect.y,
                                            anchor_rect.width,
                                            anchor_rect.height,
                                        ),
                                        anchor: settings.positioner.anchor as u32,
                                        gravity: settings.positioner.gravity as u32,
                                        offset: settings.positioner.offset,
                                        constraint_adjustment: settings
                                            .positioner
                                            .constraint_adjustment,
                                        grab: settings.grab,
                                        window_geometry,
                                    })
                                    .expect("Send control action");

                                *is_window_opening = true;
                            }
                            popup::Action::Hide { id } => {
                                // Close the popup - remove from popup_manager and destroy winit surface
                                let _ = ui_caches.remove(&id);
                                let _ = interfaces.remove(&id);

                                if let Some(removed) = popup_manager.remove_by_iced_id(id) {
                                    events.push((
                                        id,
                                        core::Event::Window(core::window::Event::Closed),
                                    ));
                                    // Actually destroy the winit popup surface
                                    if let Some(winit_id) = removed.winit_popup_id {
                                        let _ = control_sender.start_send(Control::DestroyPopup {
                                            winit_popup_id: winit_id,
                                        });
                                    }
                                } else {
                                    tracing::warn!("Popup {:?} not found in popup_manager", id);
                                }
                            }
                            popup::Action::Resize { id, width, height } => {
                                if let Some(comp) = compositor.as_mut() {
                                    if let Some((winit_id, parent_id)) =
                                        popup_manager.resize(id, width, height, comp)
                                    {
                                        let _ = control_sender.start_send(Control::ResizePopup {
                                            winit_popup_id: winit_id,
                                            width,
                                            height,
                                        });

                                        if let Some(parent) = window_manager.get_mut(parent_id) {
                                            parent.raw.request_redraw();
                                        }
                                    }
                                }
                            }
                        },
                    }
                }
            }
        }
        #[cfg(not(all(
            unix,
            not(any(target_os = "macos", target_os = "ios", target_os = "android"))
        )))]
        Action::PlatformSpecific(action) => {
            // Platform-specific actions not supported on this platform
            let _ = action;
        }
    }
}

/// Build the user interface for every window.
pub fn build_user_interfaces<'a, P: Program, C>(
    program: &'a program::Instance<P>,
    window_manager: &mut WindowManager<P, C>,
    mut cached_user_interfaces: FxHashMap<window::Id, user_interface::Cache>,
) -> FxHashMap<window::Id, UserInterface<'a, P::Message, P::Theme, P::Renderer>>
where
    C: Compositor<Renderer = P::Renderer>,
    P::Theme: theme::Base,
{
    use crate::core::Renderer as _;

    for (id, window) in window_manager.iter_mut() {
        window.state.synchronize(program, id, &window.raw);

        #[cfg(feature = "hinting")]
        window.renderer.hint(window.state.scale_factor());
    }

    debug::theme_changed(|| {
        window_manager
            .first()
            .and_then(|window| theme::Base::palette(window.state.theme()))
    });

    cached_user_interfaces
        .drain()
        .filter_map(|(id, cache)| {
            let window = window_manager.get_mut(id)?;

            Some((
                id,
                build_user_interface(
                    program,
                    cache,
                    &mut window.renderer,
                    window.state.logical_size(),
                    id,
                ),
            ))
        })
        .collect()
}

/// Returns true if the provided event should cause a [`Program`] to
/// exit.
pub fn user_force_quit(
    event: &winit::event::WindowEvent,
    _modifiers: winit::keyboard::ModifiersState,
) -> bool {
    match event {
        #[cfg(target_os = "macos")]
        winit::event::WindowEvent::KeyboardInput {
            event:
                winit::event::KeyEvent {
                    logical_key: winit::keyboard::Key::Character(c),
                    state: winit::event::ElementState::Pressed,
                    ..
                },
            ..
        } if c == "q" && _modifiers.super_key() => true,
        _ => false,
    }
}

#[cfg(feature = "sysinfo")]
fn system_information(graphics: compositor::Information) -> system::Information {
    use sysinfo::{Process, System};

    let mut system = System::new_all();
    system.refresh_all();

    let cpu_brand = system
        .cpus()
        .first()
        .map(|cpu| cpu.brand().to_string())
        .unwrap_or_default();

    let memory_used = sysinfo::get_current_pid()
        .and_then(|pid| system.process(pid).ok_or("Process not found"))
        .map(Process::memory)
        .ok();

    system::Information {
        system_name: System::name(),
        system_kernel: System::kernel_version(),
        system_version: System::long_os_version(),
        system_short_version: System::os_version(),
        cpu_brand,
        cpu_cores: system.physical_core_count(),
        memory_total: system.total_memory(),
        memory_used,
        graphics_adapter: graphics.adapter,
        graphics_backend: graphics.backend,
    }
}

/// Pre-process any [`DndIcon::Element`] variants in the clipboard's DnD
/// requests by rendering them offscreen and replacing them with
/// [`DndIcon::Pixels`].
///
/// This must be called *before* `run_clipboard`, while the compositor and
/// window state (`viewport`, `theme`, `text_color`) are still available.
fn resolve_dnd_icon_elements<P, C>(
    clipboard_requests: &mut core::Clipboard,
    compositor: &mut C,
    viewport: &graphics::Viewport,
    theme: &P::Theme,
    text_color: core::Color,
) where
    P: Program + 'static,
    P::Theme: theme::Base + 'static,
    C: Compositor<Renderer = P::Renderer>,
    P::Renderer: 'static,
{
    for req in &mut clipboard_requests.dnd_requests {
        if let core::dnd::Request::StartDrag { icon, .. } = req {
            let needs_render = matches!(icon, Some(core::dnd::DndIcon::Element(_)));
            if needs_render {
                // Take the Element icon out of the request.
                let elem_icon = match icon.take() {
                    Some(core::dnd::DndIcon::Element(surface)) => surface,
                    _ => unreachable!(),
                };

                // Downcast to the program's concrete types.
                if let Some((mut element, state)) = elem_icon.downcast::<P::Theme, P::Renderer>() {
                    // Create a fresh offscreen renderer.
                    let mut icon_renderer = compositor.create_renderer();

                    // Use 2x scale for crisp rendering on HiDPI displays.
                    // The Wayland compositor will handle scaling the surface appropriately.
                    let icon_scale = viewport.scale_factor().max(2.0);

                    // Layout the element to determine its natural size.
                    let lim = core::layout::Limits::new(
                        core::Size::new(1.0, 1.0),
                        core::Size::new(
                            viewport.physical_width() as f32,
                            viewport.physical_height() as f32,
                        ),
                    );

                    let mut tree = core::widget::Tree {
                        tag: element.as_widget().tag(),
                        state,
                        children: element.as_widget().children(),
                    };

                    let layout_node =
                        element
                            .as_widget_mut()
                            .layout(&mut tree, &icon_renderer, &lim);

                    let size = lim.resolve(
                        core::Length::Shrink,
                        core::Length::Shrink,
                        layout_node.size(),
                    );

                    // Convert logical size to physical size using our icon scale.
                    let physical = core::Size::new(
                        (size.width * icon_scale).ceil() as u32,
                        (size.height * icon_scale).ceil() as u32,
                    );
                    let icon_viewport =
                        graphics::Viewport::with_physical_size(physical, icon_scale);

                    // Build a UserInterface for the element and draw it.
                    let mut ui = UserInterface::build(
                        element,
                        size,
                        user_interface::Cache::default(),
                        &mut icon_renderer,
                    );
                    let _ = ui.draw(
                        &mut icon_renderer,
                        theme,
                        &renderer::Style { text_color },
                        core::mouse::Cursor::Unavailable,
                    );

                    // Screenshot to RGBA, then convert to pre-multiplied ARGB.
                    let mut bytes = compositor.screenshot(
                        &mut icon_renderer,
                        &icon_viewport,
                        core::Color::TRANSPARENT,
                    );
                    for pix in bytes.chunks_exact_mut(4) {
                        // RGBA → ARGB (little-endian pre-multiplied)
                        pix.swap(0, 2);
                    }

                    let w = icon_viewport.physical_width();
                    let h = icon_viewport.physical_height();
                    let scale_int = icon_scale.ceil() as i32;

                    *icon = Some(core::dnd::DndIcon::Pixels {
                        width: w,
                        height: h,
                        pixels: bytes,
                        scale: scale_int,
                    });
                } else {
                    tracing::warn!("resolve_dnd_icon_elements: failed to downcast DndIconSurface");
                }
            }
        }
    }
}

fn run_clipboard<Message: Send>(
    proxy: &mut Proxy<Message>,
    clipboard: &mut Clipboard,
    requests: core::Clipboard,
    window: window::Id,
    raw_window: Option<&winit::window::Window>,
) {
    for kind in requests.reads {
        let proxy = proxy.clone();

        clipboard.read(kind, move |result| {
            proxy.send_action(Action::Event {
                window,
                event: core::Event::Clipboard(core::clipboard::Event::Read(result.map(Arc::new))),
            });
        });
    }

    if let Some(content) = requests.write {
        let proxy = proxy.clone();

        clipboard.write(content, move |result| {
            proxy.send_action(Action::Event {
                window,
                event: core::Event::Clipboard(core::clipboard::Event::Written(result)),
            });
        });
    }

    // Process DnD requests.
    if !requests.dnd_requests.is_empty() {
        tracing::trace!(
            dnd_requests_count = requests.dnd_requests.len(),
            has_raw_window = raw_window.is_some(),
            "run_clipboard: has DnD requests to process"
        );
    }
    #[cfg(all(
        feature = "wayland",
        any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
        )
    ))]
    if !requests.dnd_requests.is_empty() {
        tracing::trace!(
            count = requests.dnd_requests.len(),
            "run_clipboard: processing DnD requests"
        );
        if let Some(raw) = raw_window {
            use winit::platform::wayland::WindowExtWayland;
            for req in requests.dnd_requests {
                match req {
                    core::dnd::Request::StartDrag {
                        mime_types,
                        actions,
                        data,
                        icon,
                        ..
                    } => {
                        let action_bits = actions.bits();
                        // By this point, Element icons should already have been
                        // resolved to Pixels by `resolve_dnd_icon_elements`.
                        let icon_data = match icon {
                            Some(core::dnd::DndIcon::Pixels {
                                width,
                                height,
                                pixels,
                                scale,
                            }) => Some((width, height, pixels, scale)),
                            Some(core::dnd::DndIcon::Element(_)) => {
                                tracing::error!(
                                    "DnD: Element icon was not resolved to pixels before run_clipboard!"
                                );
                                None
                            }
                            None => None,
                        };
                        let (ok, _mime_types, _data) =
                            raw.start_drag(mime_types, action_bits, data, icon_data);
                        if ok {
                            tracing::trace!("DnD: start_drag succeeded");
                        } else {
                            tracing::warn!("DnD: start_drag failed");
                        }
                    }
                    core::dnd::Request::AcceptMimeType(mime) => {
                        raw.dnd_accept_mime_type(mime.as_deref());
                    }
                    core::dnd::Request::SetActions { actions, preferred } => {
                        raw.dnd_set_actions(actions.bits(), preferred.bits());
                    }
                    core::dnd::Request::RequestData { mime_type } => {
                        raw.dnd_request_data(&mime_type);
                    }
                    core::dnd::Request::FinishDnd => {
                        raw.dnd_finish();
                    }
                    core::dnd::Request::EndDnd => {
                        // EndDnd is handled by the compositor when the drag finishes
                    }
                }
            }
        }
    }
}
