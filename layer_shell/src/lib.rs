//! A small Wayland layer-shell runner for Iced applications.

use std::{
    ffi::c_void,
    fmt,
    num::NonZeroU32,
    os::fd::AsRawFd,
    ptr::NonNull,
    sync::mpsc::{Receiver, TryRecvError},
};

pub use iced_core::Theme;
use iced_core::{
    Color, Event, Point, Settings, clipboard as core_clipboard, keyboard, mouse, renderer, shell,
    theme::{self, Base},
    window,
};
use iced_futures::futures::{FutureExt, StreamExt};
use iced_graphics::{Shell, Viewport};
use iced_program::Program;
use iced_futures::{Executor, Runtime, subscription};
use iced_runtime::user_interface::{self, UserInterface};
use iced_runtime::{Action, task};
pub use iced_wgpu::Renderer;
use iced_wgpu::{Engine, wgpu};
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle, WindowHandle,
};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry,
    output::{OutputHandler, OutputState},
    reexports::protocols::ext::background_effect::v1::client::{
        ext_background_effect_manager_v1::ExtBackgroundEffectManagerV1,
        ext_background_effect_surface_v1::ExtBackgroundEffectSurfaceV1,
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        Capability, SeatHandler, SeatState,
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers as SeatModifiers, RawModifiers},
        pointer::{
            CursorIcon, PointerEvent, PointerEventKind, PointerHandler, PointerThemeError,
            ThemeSpec, ThemedPointer,
        },
    },
    shell::{
        WaylandSurface,
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
    },
    shm::{Shm, ShmHandler},
};
use thiserror::Error;
use wayland_client::{
    Connection, EventQueue, Proxy, QueueHandle, delegate_noop,
    globals::registry_queue_init,
    protocol::{wl_keyboard, wl_output, wl_region::WlRegion, wl_seat, wl_surface},
};
use wayland_protocols_plasma::blur::client::{
    org_kde_kwin_blur::OrgKdeKwinBlur, org_kde_kwin_blur_manager::OrgKdeKwinBlurManager,
};

/// Errors returned while starting or running a layer-shell application.
#[derive(Debug, Error)]
pub enum RunError {
    /// Connecting to the Wayland compositor failed.
    #[error("could not connect to the Wayland compositor: {0}")]
    Connect(#[from] wayland_client::ConnectError),
    /// Creating the Wayland event queue failed.
    #[error("could not initialize the Wayland registry: {0}")]
    Registry(String),
    /// The compositor does not provide the required layer-shell protocol.
    #[error("the compositor does not provide wlr-layer-shell")]
    LayerShellUnavailable,
    /// The wgpu surface could not be created.
    #[error("could not create the wgpu surface: {0}")]
    Surface(#[from] wgpu::CreateSurfaceError),
    /// No compatible GPU adapter was found.
    #[error("no compatible GPU adapter was found")]
    AdapterUnavailable,
    /// Creating the wgpu device failed.
    #[error("could not create the wgpu device: {0}")]
    Device(#[from] wgpu::RequestDeviceError),
    /// Wayland event dispatch failed.
    #[error("Wayland event dispatch failed: {0}")]
    Dispatch(#[from] wayland_client::DispatchError),
    /// Wayland socket I/O failed.
    #[error("Wayland socket I/O failed: {0}")]
    WaylandIo(#[from] wayland_client::backend::WaylandError),
    /// Waiting for Wayland events failed.
    #[error("polling the Wayland connection failed: {0}")]
    Poll(#[from] std::io::Error),
    /// The wgpu surface has no compatible format.
    #[error("the wgpu surface has no supported texture format")]
    NoSurfaceFormat,
    /// Neither supported Wayland background-blur protocol is available.
    #[error("the compositor does not provide a supported background-blur protocol")]
    BlurUnavailable,
    /// The compositor does not expose an alpha mode needed for transparency.
    #[error("the wgpu surface does not support transparent alpha modes (available: {0})")]
    TransparencyUnavailable(String),
    /// Acquiring a frame from the wgpu surface failed.
    #[error("could not acquire a frame from the wgpu surface")]
    SurfaceFrameUnavailable,
    /// An application requested a task action this runner does not implement.
    #[error("the layer-shell runner does not support this Iced task action")]
    UnsupportedTask,
    /// An application started a task that requires asynchronous execution.
    #[error("the layer-shell runner cannot execute asynchronous Iced tasks")]
    AsyncTask,
    /// Initializing or updating the Wayland cursor failed.
    #[error("could not initialize or update the Wayland cursor: {0}")]
    Cursor(String),
    /// Initializing the Wayland keyboard failed.
    #[error("could not initialize the Wayland keyboard: {0}")]
    Keyboard(String),
    /// Creating the asynchronous executor failed.
    #[error("could not create the asynchronous executor: {0}")]
    Executor(String),
}

/// Result type for the layer-shell runner.
pub type Result<T = (), E = RunError> = std::result::Result<T, E>;

/// A command that controls the visibility of a layer-shell window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibilityCommand {
    /// Raise the layer surface and redraw it.
    Show,
    /// Move the layer surface behind normal windows.
    Hide,
}

/// Runs an Iced program on a Wayland layer surface.
///
/// This runner is intended for simple single-surface applications. It supports
/// pointer input and immediate Iced task actions, including `iced::exit()`.
pub fn run<P>(
    program: P,
    settings: Settings,
    window_settings: window::Settings,
) -> Result<(), RunError>
where
    P: Program<Renderer = Renderer, Theme = iced_core::Theme> + 'static,
{
    run_inner(program, settings, window_settings, None)
}

/// Runs an Iced program and accepts commands to show or hide its layer surface.
pub fn run_with_visibility<P>(
    program: P,
    settings: Settings,
    window_settings: window::Settings,
    commands: Receiver<VisibilityCommand>,
) -> Result<(), RunError>
where
    P: Program<Renderer = Renderer, Theme = iced_core::Theme> + 'static,
{
    run_inner(program, settings, window_settings, Some(commands))
}

fn run_inner<P>(
    program: P,
    settings: Settings,
    window_settings: window::Settings,
    mut visibility_commands: Option<Receiver<VisibilityCommand>>,
) -> Result<(), RunError>
where
    P: Program<Renderer = Renderer, Theme = iced_core::Theme> + 'static,
{
    let connection = Connection::connect_to_env()?;
    let (globals, mut event_queue) =
        registry_queue_init(&connection).map_err(|error| RunError::Registry(error.to_string()))?;
    let queue_handle = event_queue.handle();
    let seat_state = SeatState::new(&globals, &queue_handle);
    let shm = Shm::bind(&globals, &queue_handle)
        .map_err(|error| RunError::Registry(error.to_string()))?;

    let compositor =
        CompositorState::bind(&globals, &queue_handle).expect("wl_compositor is unavailable");
    let layer_shell =
        LayerShell::bind(&globals, &queue_handle).map_err(|_| RunError::LayerShellUnavailable)?;
    let surface = compositor.create_surface(&queue_handle);
    let layer =
        layer_shell.create_layer_surface(&queue_handle, surface, Layer::Top, Some(P::name()), None);
    layer.set_anchor(Anchor::empty());
    layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
    let initial_width = window_settings.size.width.max(1.0) as u32;
    let initial_height = window_settings.size.height.max(1.0) as u32;
    layer.set_size(initial_width, initial_height);
    apply_blur(
        &globals,
        &queue_handle,
        &compositor,
        &layer,
        initial_width,
        initial_height,
    )?;
    layer.commit();

    let mut host = Host {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &queue_handle),
        seat_state,
        compositor_state: compositor.clone(),
        shm,
        clipboard: None,
        clipboard_init_error: None,
        cursor_error: None,
        keyboard_error: None,
        pointer: None,
        keyboard: None,
        keyboard_focus: false,
        keyboard_modifiers: keyboard::Modifiers::empty(),
        cursor_interaction: None,
        events: Vec::new(),
        cursor: mouse::Cursor::Unavailable,
        layer,
        width: initial_width,
        height: initial_height,
        configured: false,
        closed: false,
        input_passthrough: false,
    };

    let mut visible = true;
    while !host.configured && !host.closed {
        dispatch_events(&mut event_queue, &mut host, visibility_commands.is_some())?;
        check_cursor_error(&mut host)?;
        check_keyboard_error(&mut host)?;
        check_keyboard_error(&mut host)?;
        for command in drain_visibility_commands(&mut visibility_commands) {
            visible = command == VisibilityCommand::Show;
        }
    }

    if host.closed {
        return Ok(());
    }
    if !visible {
        host.layer.set_layer(Layer::Background);
        host.set_empty_input_region(&queue_handle);
        host.layer.commit();
    }

    let handles = WaylandHandles {
        connection: connection.clone(),
    };
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(
        Box::new(handles),
    ));
    let wayland_window = WaylandWindow {
        connection: connection.clone(),
        surface: host.layer.wl_surface().clone(),
    };
    let surface = instance.create_surface(wgpu::SurfaceTarget::Window(Box::new(
        wayland_window.clone(),
    )))?;
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
    }))
    .map_err(|_| RunError::AdapterUnavailable)?;
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default()))?;

    let capabilities = surface.get_capabilities(&adapter);
    let format = capabilities
        .formats
        .iter()
        .copied()
        .find(wgpu::TextureFormat::is_srgb)
        .or_else(|| capabilities.formats.first().copied())
        .ok_or(RunError::NoSurfaceFormat)?;
    let alpha_mode = [
        wgpu::CompositeAlphaMode::PreMultiplied,
        wgpu::CompositeAlphaMode::PostMultiplied,
    ]
    .into_iter()
    .find(|mode| capabilities.alpha_modes.contains(mode))
    .ok_or_else(|| RunError::TransparencyUnavailable(format!("{:?}", capabilities.alpha_modes)))?;
    let physical_size = iced_core::Size::new(host.width, host.height);

    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: host.width.max(1),
        height: host.height.max(1),
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
        color_space: wgpu::SurfaceColorSpace::Auto,
    };
    surface.configure(&device, &surface_config);

    let engine = Engine::new(
        &adapter,
        device.clone(),
        queue.clone(),
        format,
        None,
        Shell::headless(),
    );
    let mut renderer = Renderer::new(engine, renderer::Settings::from(&settings));
    let (mut program, boot_task) = iced_program::Instance::new(program);
    let executor = P::Executor::new().map_err(|error| RunError::Executor(error.to_string()))?;
    let (runtime_sender, mut runtime_receiver) =
        iced_futures::futures::channel::mpsc::unbounded();
    let mut runtime = Runtime::new(executor, runtime_sender);
    let window_id = window::Id::unique();
    let viewport = Viewport::with_physical_size(
        physical_size,
        renderer::Scale {
            window: 1.0,
            application: program.scale_factor(window_id),
        },
    );
    let theme = program
        .theme(window_id)
        .unwrap_or_else(|| <iced_core::Theme as Base>::default(theme::Mode::None));
    let mut cache = user_interface::Cache::default();
    process_task(boot_task, &mut program, &mut host)?;
    let current_subscription =
        runtime.enter(|| program.subscription().map(Action::Output));
    runtime.track(subscription::into_recipes(current_subscription));
    if visible {
        draw_frame(
            &program,
            window_id,
            &theme,
            &viewport,
            &mut cache,
            &mut renderer,
            &queue,
            &surface,
            host.cursor,
        )?;
    }

    let waker = shell::Waker::noop();
    let mut event_buffer = Vec::new();
    while !host.closed {
        dispatch_events(&mut event_queue, &mut host, true)?;
        check_cursor_error(&mut host)?;
        check_keyboard_error(&mut host)?;
        let subscription_updated =
            drain_runtime_actions(&mut runtime_receiver, &mut program, &mut host)?;
        if subscription_updated {
            let current_subscription =
                runtime.enter(|| program.subscription().map(Action::Output));
            runtime.track(subscription::into_recipes(current_subscription));
        }
        for command in drain_visibility_commands(&mut visibility_commands) {
            match command {
                VisibilityCommand::Hide if visible => {
                    host.layer.set_layer(Layer::Background);
                    host.layer
                        .set_keyboard_interactivity(KeyboardInteractivity::None);
                    host.input_passthrough = true;
                    host.set_empty_input_region(&queue_handle);
                    host.layer.commit();
                    host.cursor_interaction = None;
                    host.keyboard_focus = false;
                    host.events.clear();
                    visible = false;
                }
                VisibilityCommand::Show if !visible => {
                    host.layer.set_layer(Layer::Top);
                    host.layer
                        .set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
                    host.input_passthrough = false;
                    host.set_rounded_input_region(&queue_handle);
                    host.layer.commit();
                    visible = true;
                    draw_frame(
                        &program,
                        window_id,
                        &theme,
                        &viewport,
                        &mut cache,
                        &mut renderer,
                        &queue,
                        &surface,
                        host.cursor,
                    )?;
                }
                VisibilityCommand::Show | VisibilityCommand::Hide => {}
            }
        }
        if host.closed || !visible {
            continue;
        }
        if host.events.is_empty() {
            if subscription_updated {
                draw_frame(
                    &program,
                    window_id,
                    &theme,
                    &viewport,
                    &mut cache,
                    &mut renderer,
                    &queue,
                    &surface,
                    host.cursor,
                )?;
            }
            continue;
        }
        event_buffer.append(&mut host.events);
        let mut interface = UserInterface::build(
            program.view(window_id),
            viewport.logical_size(),
            cache,
            &mut renderer,
        );
        let mut messages = shell::Bus::new();
        let (ui_state, event_statuses) = interface.update(
            &wayland_window,
            &waker,
            &event_buffer,
            host.cursor,
            &mut renderer,
            &mut messages,
        );
        let clipboard_requests = match ui_state {
            user_interface::State::Updated {
                mouse_interaction,
                clipboard,
                ..
            } => {
                host.set_cursor(&connection, mouse_interaction)
                    .map_err(RunError::Cursor)?;
                Some(clipboard)
            }
            user_interface::State::Outdated => None,
        };
        for (event, status) in event_buffer.iter().cloned().zip(event_statuses) {
            runtime.broadcast(subscription::Event::Interaction {
                window: window_id,
                event,
                status,
            });
        }
        if let Some(requests) = clipboard_requests {
            handle_clipboard_requests(&mut host, requests);
        }
        cache = interface.into_cache();
        event_buffer.clear();
        process_messages(messages, &mut program, &mut host)?;
        let current_subscription =
            runtime.enter(|| program.subscription().map(Action::Output));
        runtime.track(subscription::into_recipes(current_subscription));
        if !host.closed {
            draw_frame(
                &program,
                window_id,
                &theme,
                &viewport,
                &mut cache,
                &mut renderer,
                &queue,
                &surface,
                host.cursor,
            )?;
        }
    }

    Ok(())
}

fn drain_visibility_commands(
    commands: &mut Option<Receiver<VisibilityCommand>>,
) -> Vec<VisibilityCommand> {
    let Some(receiver) = commands.as_ref() else {
        return Vec::new();
    };
    let mut received = Vec::new();
    let mut disconnected = false;

    loop {
        match receiver.try_recv() {
            Ok(command) => received.push(command),
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => {
                disconnected = true;
                break;
            }
        }
    }

    if disconnected {
        *commands = None;
    }

    received
}

fn handle_clipboard_requests(host: &mut Host, requests: core_clipboard::Clipboard) {
    for kind in requests.reads {
        let result = if kind == core_clipboard::Kind::Text {
            get_clipboard(host).and_then(|clipboard| {
                clipboard
                    .get_text()
                    .map(core_clipboard::Content::Text)
                    .map_err(clipboard_error)
            })
        } else {
            Err(core_clipboard::Error::ClipboardUnavailable)
        };

        if let Err(error) = &result {
            eprintln!("clipboard read failed: {error:?}");
        }
        host.events.push(Event::Clipboard(core_clipboard::Event::Read(
            result.map(std::sync::Arc::new),
        )));
    }

    if let Some(content) = requests.write {
        let result = match content {
            core_clipboard::Content::Text(text) => get_clipboard(host)
                .and_then(|clipboard| clipboard.set_text(text).map_err(clipboard_error)),
            _ => Err(core_clipboard::Error::ClipboardUnavailable),
        };

        if let Err(error) = &result {
            eprintln!("clipboard write failed: {error:?}");
        }
        host.events
            .push(Event::Clipboard(core_clipboard::Event::Written(result)));
    }
}

fn get_clipboard(host: &mut Host) -> Result<&mut arboard::Clipboard, core_clipboard::Error> {
    if host.clipboard.is_none() && host.clipboard_init_error.is_none() {
        match arboard::Clipboard::new() {
            Ok(clipboard) => host.clipboard = Some(clipboard),
            Err(error) => {
                eprintln!("could not initialize the Wayland clipboard: {error}");
                host.clipboard_init_error = Some(error.to_string());
            }
        }
    }

    host.clipboard
        .as_mut()
        .ok_or(core_clipboard::Error::ClipboardUnavailable)
}

fn clipboard_error(error: arboard::Error) -> core_clipboard::Error {
    match error {
        arboard::Error::ContentNotAvailable => core_clipboard::Error::ContentNotAvailable,
        arboard::Error::ClipboardNotSupported => core_clipboard::Error::ClipboardUnavailable,
        arboard::Error::ClipboardOccupied => core_clipboard::Error::ClipboardOccupied,
        arboard::Error::ConversionFailure => core_clipboard::Error::ConversionFailure,
        arboard::Error::Unknown { description } => core_clipboard::Error::Unknown {
            description: std::sync::Arc::new(description),
        },
        error => core_clipboard::Error::Unknown {
            description: std::sync::Arc::new(error.to_string()),
        },
    }
}

#[allow(unsafe_code)]
fn dispatch_events(
    event_queue: &mut EventQueue<Host>,
    host: &mut Host,
    poll_for_commands: bool,
) -> Result<(), RunError> {
    if !poll_for_commands {
        let _ = event_queue.blocking_dispatch(host)?;
        return Ok(());
    }

    loop {
        let _ = event_queue.dispatch_pending(host)?;
        let Some(read_guard) = event_queue.prepare_read() else {
            continue;
        };
        event_queue.flush()?;

        let mut descriptor = libc::pollfd {
            fd: read_guard.connection_fd().as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        let ready = unsafe { libc::poll(&mut descriptor, 1, 50) };
        if ready < 0 {
            let error = std::io::Error::last_os_error();
            drop(read_guard);
            if error.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return Err(RunError::Poll(error));
        }

        if ready > 0 && descriptor.revents & (libc::POLLIN | libc::POLLERR | libc::POLLHUP) != 0 {
            let _ = read_guard.read()?;
        } else {
            drop(read_guard);
        }

        let _ = event_queue.dispatch_pending(host)?;
        return Ok(());
    }
}

fn check_cursor_error(host: &mut Host) -> Result<(), RunError> {
    match host.cursor_error.take() {
        Some(error) => Err(RunError::Cursor(error)),
        None => Ok(()),
    }
}

fn check_keyboard_error(host: &mut Host) -> Result<(), RunError> {
    match host.keyboard_error.take() {
        Some(error) => Err(RunError::Keyboard(error)),
        None => Ok(()),
    }
}

fn draw_frame<P: Program<Renderer = Renderer, Theme = iced_core::Theme>>(
    program: &iced_program::Instance<P>,
    window_id: window::Id,
    theme: &iced_core::Theme,
    viewport: &Viewport,
    cache: &mut user_interface::Cache,
    renderer: &mut Renderer,
    queue: &wgpu::Queue,
    surface: &wgpu::Surface<'_>,
    cursor: mouse::Cursor,
) -> Result<(), RunError> {
    let mut interface = UserInterface::build(
        program.view(window_id),
        viewport.logical_size(),
        std::mem::take(cache),
        renderer,
    );
    let frame = match surface.get_current_texture() {
        wgpu::CurrentSurfaceTexture::Success(frame)
        | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
        _ => return Err(RunError::SurfaceFrameUnavailable),
    };
    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let app_style = program.style(theme);
    interface.draw(
        renderer,
        theme,
        &renderer::Style {
            text_color: app_style.text_color,
        },
        cursor,
    );
    let _submission = renderer.present(
        Some(Color::TRANSPARENT),
        frame.texture.format(),
        &view,
        &viewport,
    );
    *cache = interface.into_cache();
    queue.present(frame);
    Ok(())
}

fn process_task<P: Program<Renderer = Renderer, Theme = iced_core::Theme>>(
    task: iced_runtime::Task<P::Message>,
    program: &mut iced_program::Instance<P>,
    host: &mut Host,
) -> Result<(), RunError> {
    let Some(mut actions) = task::into_stream(task) else {
        return Ok(());
    };

    loop {
        match actions.next().now_or_never() {
            Some(Some(Action::Output(message))) => {
                process_task(program.update(message), program, host)?;
            }
            Some(Some(Action::Exit)) => {
                host.closed = true;
            }
            Some(Some(_)) => return Err(RunError::UnsupportedTask),
            Some(None) => return Ok(()),
            None => return Err(RunError::AsyncTask),
        }
        if host.closed {
            return Ok(());
        }
    }
}

fn drain_runtime_actions<P: Program<Renderer = Renderer, Theme = iced_core::Theme>>(
    receiver: &mut iced_futures::futures::channel::mpsc::UnboundedReceiver<Action<P::Message>>,
    program: &mut iced_program::Instance<P>,
    host: &mut Host,
) -> Result<bool, RunError> {
    let mut updated = false;

    loop {
        match receiver.try_recv() {
            Ok(Action::Output(message)) => {
                process_task(program.update(message), program, host)?;
                updated = true;
            }
            Ok(Action::Exit) => {
                host.closed = true;
                updated = true;
            }
            Ok(_) => return Err(RunError::UnsupportedTask),
            Err(_) => break,
        }

        if host.closed {
            break;
        }
    }

    Ok(updated)
}

fn process_messages<P: Program<Renderer = Renderer, Theme = iced_core::Theme>>(
    mut messages: shell::Bus<P::Message>,
    program: &mut iced_program::Instance<P>,
    host: &mut Host,
) -> Result<(), RunError> {
    for (message, _) in messages.drain() {
        process_task(program.update(message), program, host)?;
        if host.closed {
            break;
        }
    }
    Ok(())
}

fn apply_blur(
    globals: &wayland_client::globals::GlobalList,
    queue_handle: &QueueHandle<Host>,
    compositor: &CompositorState,
    layer: &LayerSurface,
    width: u32,
    height: u32,
) -> Result<(), RunError> {
    if let Ok(manager) = globals.bind::<ExtBackgroundEffectManagerV1, _, _>(queue_handle, 1..=1, ())
    {
        let effect = manager.get_background_effect(layer.wl_surface(), queue_handle, ());
        let region = compositor.wl_compositor().create_region(queue_handle, ());
        region.add(0, 0, width as i32, height as i32);
        effect.set_blur_region(Some(&region));
        region.destroy();
        return Ok(());
    }

    if let Ok(manager) = globals.bind::<OrgKdeKwinBlurManager, _, _>(queue_handle, 1..=1, ()) {
        let effect = manager.create(layer.wl_surface(), queue_handle, ());
        let region = compositor.wl_compositor().create_region(queue_handle, ());
        region.add(0, 0, width as i32, height as i32);
        effect.set_region(Some(&region));
        effect.commit();
        region.destroy();
        return Ok(());
    }

    Err(RunError::BlurUnavailable)
}

struct Host {
    registry_state: RegistryState,
    output_state: OutputState,
    seat_state: SeatState,
    compositor_state: CompositorState,
    shm: Shm,
    clipboard: Option<arboard::Clipboard>,
    clipboard_init_error: Option<String>,
    cursor_error: Option<String>,
    keyboard_error: Option<String>,
    pointer: Option<ThemedPointer>,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    keyboard_focus: bool,
    keyboard_modifiers: keyboard::Modifiers,
    cursor_interaction: Option<mouse::Interaction>,
    events: Vec<Event>,
    cursor: mouse::Cursor,
    layer: LayerSurface,
    width: u32,
    height: u32,
    configured: bool,
    closed: bool,
    input_passthrough: bool,
}

impl fmt::Debug for Host {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Host")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("configured", &self.configured)
            .field("closed", &self.closed)
            .finish()
    }
}

impl CompositorHandler for Host {
    fn scale_factor_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: wl_output::Transform,
    ) {
    }

    fn frame(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {}

    fn surface_enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }
}

impl LayerShellHandler for Host {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        self.closed = true;
    }

    fn configure(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        _: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _: u32,
    ) {
        self.width = NonZeroU32::new(configure.new_size.0).map_or(self.width, NonZeroU32::get);
        self.height = NonZeroU32::new(configure.new_size.1).map_or(self.height, NonZeroU32::get);
        if self.input_passthrough {
            self.set_empty_input_region(qh);
        } else {
            self.set_rounded_input_region(qh);
        }
        self.configured = true;
    }
}

impl Host {
    fn set_empty_input_region(&self, qh: &QueueHandle<Self>) {
        let region = self.compositor_state.wl_compositor().create_region(qh, ());
        self.layer.wl_surface().set_input_region(Some(&region));
        region.destroy();
    }

    fn set_rounded_input_region(&self, qh: &QueueHandle<Self>) {
        let width = self.width;
        let height = self.height;
        let radius = 22_u32.min(width / 2).min(height / 2);
        let region = self.compositor_state.wl_compositor().create_region(qh, ());

        for y in 0..height {
            let inset = rounded_corner_inset(y, width, height, radius);
            let span = width.saturating_sub(2 * inset);
            if span > 0 {
                region.add(inset as i32, y as i32, span as i32, 1);
            }
        }

        self.layer.wl_surface().set_input_region(Some(&region));
        region.destroy();
    }
}

fn rounded_corner_inset(y: u32, width: u32, height: u32, radius: u32) -> u32 {
    if radius == 0 || (y >= radius && y < height.saturating_sub(radius)) {
        return 0;
    }

    let row_center = if y < radius {
        y as f64 + 0.5
    } else {
        height as f64 - y as f64 - 0.5
    };
    let distance = radius as f64 - row_center;
    let remaining = (radius as f64).powi(2) - distance.powi(2);
    (radius as f64 - remaining.max(0.0).sqrt())
        .ceil()
        .clamp(0.0, (width / 2) as f64) as u32
}

#[cfg(test)]
mod tests {
    use super::rounded_corner_inset;

    #[test]
    fn rounded_input_region_is_symmetric() {
        let (width, height, radius) = (560, 300, 22);

        for y in 0..height {
            assert_eq!(
                rounded_corner_inset(y, width, height, radius),
                rounded_corner_inset(height - y - 1, width, height, radius),
            );
        }

        assert_eq!(rounded_corner_inset(radius, width, height, radius), 0);
    }

    #[test]
    fn zero_radius_preserves_rectangular_input_region() {
        assert_eq!(rounded_corner_inset(0, 560, 300, 0), 0);
        assert_eq!(rounded_corner_inset(299, 560, 300, 0), 0);
    }
}

impl OutputHandler for Host {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl SeatHandler for Host {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer && self.pointer.is_none() {
            let shm = self.shm.wl_shm().clone();
            let cursor_surface = self.compositor_state.create_surface(qh);
            match self.seat_state.get_pointer_with_theme(
                qh,
                &seat,
                &shm,
                cursor_surface,
                ThemeSpec::System,
            ) {
                Ok(pointer) => {
                    self.pointer = Some(pointer);
                }
                Err(error) => self.cursor_error = Some(error.to_string()),
            }
        }
        if capability == Capability::Keyboard && self.keyboard.is_none() {
            match self.seat_state.get_keyboard(qh, &seat, None) {
                Ok(keyboard) => self.keyboard = Some(keyboard),
                Err(error) => self.keyboard_error = Some(error.to_string()),
            }
        }
    }

    fn remove_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer {
            self.pointer = None;
            self.cursor_interaction = None;
            self.cursor = mouse::Cursor::Unavailable;
        }
        if capability == Capability::Keyboard {
            if let Some(keyboard) = self.keyboard.take() {
                keyboard.release();
            }
            self.keyboard_focus = false;
            self.keyboard_modifiers = keyboard::Modifiers::empty();
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for Host {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
        _: &[u32],
        _: &[Keysym],
    ) {
        if surface == self.layer.wl_surface() {
            self.keyboard_focus = true;
            self.events
                .push(Event::Window(window::Event::Focused));
        }
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
    ) {
        if surface == self.layer.wl_surface() {
            self.keyboard_focus = false;
            self.events
                .push(Event::Window(window::Event::Unfocused));
        }
    }

    fn press_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        if self.keyboard_focus {
            self.events.push(Event::Keyboard(iced_key_event(
                event,
                self.keyboard_modifiers,
                false,
            )));
        }
    }

    fn repeat_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        if self.keyboard_focus {
            self.events.push(Event::Keyboard(iced_key_event(
                event,
                self.keyboard_modifiers,
                true,
            )));
        }
    }

    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        if self.keyboard_focus {
            self.events.push(Event::Keyboard(iced_key_released(
                event,
                self.keyboard_modifiers,
            )));
        }
    }

    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        modifiers: SeatModifiers,
        _: RawModifiers,
        _: u32,
    ) {
        self.keyboard_modifiers = iced_modifiers(modifiers);
        if self.keyboard_focus {
            self.events.push(Event::Keyboard(
                keyboard::Event::ModifiersChanged(self.keyboard_modifiers),
            ));
        }
    }
}

impl ShmHandler for Host {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl PointerHandler for Host {
    fn pointer_frame(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wayland_client::protocol::wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        for event in events {
            if event.surface != *self.layer.wl_surface() {
                continue;
            }
            let position = Point::new(event.position.0 as f32, event.position.1 as f32);
            match event.kind {
                PointerEventKind::Enter { .. } => {
                    self.cursor_interaction = None;
                    self.cursor = mouse::Cursor::Available(position);
                    self.events.push(Event::Mouse(mouse::Event::CursorEntered));
                    self.events
                        .push(Event::Mouse(mouse::Event::CursorMoved { position }));
                }
                PointerEventKind::Leave { .. } => {
                    self.cursor_interaction = None;
                    self.cursor = mouse::Cursor::Unavailable;
                    self.events.push(Event::Mouse(mouse::Event::CursorLeft));
                }
                PointerEventKind::Motion { .. } => {
                    self.cursor = mouse::Cursor::Available(position);
                    self.events
                        .push(Event::Mouse(mouse::Event::CursorMoved { position }));
                }
                PointerEventKind::Press { button, .. } => {
                    if let Some(button) = iced_button(button) {
                        self.events
                            .push(Event::Mouse(mouse::Event::ButtonPressed(button)));
                    }
                }
                PointerEventKind::Release { button, .. } => {
                    if let Some(button) = iced_button(button) {
                        self.events
                            .push(Event::Mouse(mouse::Event::ButtonReleased(button)));
                    }
                }
                PointerEventKind::Axis {
                    horizontal,
                    vertical,
                    ..
                } => {
                    if !horizontal.is_none() || !vertical.is_none() {
                        self.events.push(Event::Mouse(mouse::Event::WheelScrolled {
                            delta: mouse::ScrollDelta::Pixels {
                                x: horizontal.absolute as f32,
                                y: vertical.absolute as f32,
                            },
                        }));
                    }
                }
            }
        }
    }
}

impl Host {
    fn set_cursor(
        &mut self,
        connection: &Connection,
        interaction: mouse::Interaction,
    ) -> std::result::Result<(), String> {
        if self.cursor == mouse::Cursor::Unavailable {
            self.cursor_interaction = None;
            return Ok(());
        }
        if self.cursor_interaction == Some(interaction) {
            return Ok(());
        }
        if let Some(pointer) = &self.pointer {
            let result = if interaction == mouse::Interaction::Hidden {
                pointer.hide_cursor()
            } else {
                pointer.set_cursor(connection, cursor_icon(interaction))
            };
            match result {
                Ok(()) => {}
                Err(PointerThemeError::MissingEnterSerial) => return Ok(()),
                Err(error) => return Err(error.to_string()),
            }
        }
        self.cursor_interaction = Some(interaction);
        Ok(())
    }
}

fn cursor_icon(interaction: mouse::Interaction) -> CursorIcon {
    use mouse::Interaction;

    match interaction {
        Interaction::Hidden | Interaction::None | Interaction::Idle => CursorIcon::Default,
        Interaction::ContextMenu => CursorIcon::ContextMenu,
        Interaction::Help => CursorIcon::Help,
        Interaction::Pointer => CursorIcon::Pointer,
        Interaction::Progress => CursorIcon::Progress,
        Interaction::Wait => CursorIcon::Wait,
        Interaction::Cell => CursorIcon::Cell,
        Interaction::Crosshair => CursorIcon::Crosshair,
        Interaction::Text => CursorIcon::Text,
        Interaction::Alias => CursorIcon::Alias,
        Interaction::Copy => CursorIcon::Copy,
        Interaction::Move => CursorIcon::Move,
        Interaction::NoDrop => CursorIcon::NoDrop,
        Interaction::NotAllowed => CursorIcon::NotAllowed,
        Interaction::Grab => CursorIcon::Grab,
        Interaction::Grabbing => CursorIcon::Grabbing,
        Interaction::ResizingHorizontally => CursorIcon::EwResize,
        Interaction::ResizingVertically => CursorIcon::NsResize,
        Interaction::ResizingDiagonallyUp => CursorIcon::NeswResize,
        Interaction::ResizingDiagonallyDown => CursorIcon::NwseResize,
        Interaction::ResizingColumn => CursorIcon::ColResize,
        Interaction::ResizingRow => CursorIcon::RowResize,
        Interaction::AllScroll => CursorIcon::AllScroll,
        Interaction::ZoomIn => CursorIcon::ZoomIn,
        Interaction::ZoomOut => CursorIcon::ZoomOut,
    }
}

fn iced_button(button: u32) -> Option<mouse::Button> {
    Some(match button {
        0x110 => mouse::Button::Left,
        0x111 => mouse::Button::Right,
        0x112 => mouse::Button::Middle,
        0x116 => mouse::Button::Back,
        0x115 => mouse::Button::Forward,
        button => mouse::Button::Other(u16::try_from(button).ok()?),
    })
}

fn iced_key(keysym: Keysym, text: Option<&str>) -> keyboard::Key {
    use keyboard::key::{Key, Named};

    let named = match keysym {
        Keysym::Alt_L | Keysym::Alt_R => Some(Named::Alt),
        Keysym::Caps_Lock => Some(Named::CapsLock),
        Keysym::Control_L | Keysym::Control_R => Some(Named::Control),
        Keysym::Delete => Some(Named::Delete),
        Keysym::Down => Some(Named::ArrowDown),
        Keysym::End => Some(Named::End),
        Keysym::Escape => Some(Named::Escape),
        Keysym::Home => Some(Named::Home),
        Keysym::Insert => Some(Named::Insert),
        Keysym::Left => Some(Named::ArrowLeft),
        Keysym::Meta_L | Keysym::Meta_R | Keysym::Super_L | Keysym::Super_R => {
            Some(Named::Super)
        }
        Keysym::Next => Some(Named::PageDown),
        Keysym::Num_Lock => Some(Named::NumLock),
        Keysym::Prior => Some(Named::PageUp),
        Keysym::Return | Keysym::KP_Enter => Some(Named::Enter),
        Keysym::Right => Some(Named::ArrowRight),
        Keysym::Shift_L | Keysym::Shift_R => Some(Named::Shift),
        Keysym::space => Some(Named::Space),
        Keysym::Tab => Some(Named::Tab),
        Keysym::Up => Some(Named::ArrowUp),
        Keysym::BackSpace => Some(Named::Backspace),
        _ => None,
    };

    if let Some(named) = named {
        return Key::Named(named);
    }
    if let Some(character) = keysym.key_char() {
        return Key::Character(character.to_string().into());
    }
    text.filter(|text| !text.is_empty())
        .map(|text| Key::Character(text.into()))
        .unwrap_or(Key::Unidentified)
}

fn iced_key_event(
    event: KeyEvent,
    modifiers: keyboard::Modifiers,
    repeat: bool,
) -> keyboard::Event {
    let key = iced_key(event.keysym, event.utf8.as_deref());
    keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Unidentified(
            keyboard::key::NativeCode::Xkb(event.raw_code),
        ),
        location: keyboard::Location::Standard,
        modifiers,
        text: event.utf8.filter(|text| !text.is_empty()).map(Into::into),
        repeat,
    }
}

fn iced_key_released(
    event: KeyEvent,
    modifiers: keyboard::Modifiers,
) -> keyboard::Event {
    let key = iced_key(event.keysym, None);
    keyboard::Event::KeyReleased {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Unidentified(
            keyboard::key::NativeCode::Xkb(event.raw_code),
        ),
        location: keyboard::Location::Standard,
        modifiers,
    }
}

fn iced_modifiers(modifiers: SeatModifiers) -> keyboard::Modifiers {
    let mut iced_modifiers = keyboard::Modifiers::empty();
    iced_modifiers.set(keyboard::Modifiers::SHIFT, modifiers.shift);
    iced_modifiers.set(keyboard::Modifiers::CTRL, modifiers.ctrl);
    iced_modifiers.set(keyboard::Modifiers::ALT, modifiers.alt);
    iced_modifiers.set(keyboard::Modifiers::LOGO, modifiers.logo);
    iced_modifiers
}

delegate_compositor!(Host);
delegate_layer!(Host);
delegate_output!(Host);
delegate_registry!(Host);
smithay_client_toolkit::delegate_seat!(Host);
smithay_client_toolkit::delegate_pointer!(Host);
smithay_client_toolkit::delegate_keyboard!(Host);
smithay_client_toolkit::delegate_shm!(Host);
delegate_noop!(Host: ignore WlRegion);
delegate_noop!(Host: ignore ExtBackgroundEffectManagerV1);
delegate_noop!(Host: ignore ExtBackgroundEffectSurfaceV1);
delegate_noop!(Host: ignore OrgKdeKwinBlurManager);
delegate_noop!(Host: ignore OrgKdeKwinBlur);

impl ProvidesRegistryState for Host {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers!(OutputState, SeatState);
}

struct WaylandHandles {
    connection: Connection,
}

#[derive(Clone)]
struct WaylandWindow {
    connection: Connection,
    surface: wl_surface::WlSurface,
}

impl fmt::Debug for WaylandHandles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WaylandHandles").finish_non_exhaustive()
    }
}

impl fmt::Debug for WaylandWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WaylandWindow").finish_non_exhaustive()
    }
}

impl HasDisplayHandle for WaylandHandles {
    #[allow(unsafe_code)]
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let Some(display) = NonNull::new(self.connection.backend().display_ptr() as *mut c_void)
        else {
            return Err(HandleError::NotSupported);
        };
        let raw = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(display));
        // The connection outlives every surface and wgpu object created by this runner.
        Ok(unsafe { DisplayHandle::borrow_raw(raw) })
    }
}

impl HasDisplayHandle for WaylandWindow {
    #[allow(unsafe_code)]
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let Some(display) = NonNull::new(self.connection.backend().display_ptr() as *mut c_void)
        else {
            return Err(HandleError::NotSupported);
        };
        let raw = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(display));
        // The connection outlives every surface and wgpu object created by this runner.
        Ok(unsafe { DisplayHandle::borrow_raw(raw) })
    }
}

impl HasWindowHandle for WaylandWindow {
    #[allow(unsafe_code)]
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let Some(surface) = NonNull::new(self.surface.id().as_ptr() as *mut c_void) else {
            return Err(HandleError::NotSupported);
        };
        let raw = RawWindowHandle::Wayland(WaylandWindowHandle::new(surface));
        // The layer surface remains alive for the lifetime of the returned handle.
        Ok(unsafe { WindowHandle::borrow_raw(raw) })
    }
}
