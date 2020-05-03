use smithay_client_toolkit::{environment::Environment, window as sctk, reexports::client::protocol::wl_surface::WlSurface};
use iced_native::{UserInterface, Cache, window::Backend, Event, trace::{Trace, Component::{Layout, Draw, Render}}};
use super::{async_sctk::{DispatchData, Update, Item, State, Env}, application::{Application, Mode}};

///
#[derive(Debug)]
pub struct Settings {
    ///
    pub size: [u32; 2],
    ///
    pub resizable: bool,
    ///
    pub decorations: bool,
    ///
    pub overlay: bool,
}
impl Default for Settings { fn default() -> Self { Self{ size: [0,0], resizable: true, decorations: true, overlay: false } } }

pub(crate) struct Window<B:Backend> {
    pub window: Option<sctk::Window<sctk::ConceptFrame>>,
    pub size: [u32; 2], pub scale_factor: u32,
    pub cursor: &'static str,

    title: String,
    mode: Mode,
    pub backend : B, renderer : B::Renderer, pub surface: B::Surface, pub swap_chain: B::SwapChain,
    pub buffer_size: [u32; 2], pub buffer_scale_factor: u32, // fixme: should be in swap_chain
    cache: Option<Cache>
}

impl<B:Backend> Window<B> {
    pub fn new<A:Application+'static>(env: Environment<Env>, settings: self::Settings, backend: B::Settings) -> Self {
        let surface = env.create_surface_with_scale_callback(
            |scale, surface, mut data| {
                let DispatchData::<A>{state:State{window, ..}, ..} = data.get().unwrap();
                surface.set_buffer_scale(scale);
                window.scale_factor = scale as u32;
            }
        );

        use futures::stream::{LocalBoxStream, SelectAll};
        fn quit<M:'static>(streams: &mut SelectAll<LocalBoxStream<'_, Item<M>>>) {
            use futures::stream::{StreamExt, iter};
            streams.push(iter(std::iter::once(super::Item::Quit)).boxed_local())
        }

        let size = settings.size;
        let window = if settings.overlay {
            use smithay_client_toolkit::{
                reexports::protocols::wlr::unstable::layer_shell::v1::client::{
                    zwlr_layer_shell_v1::{self as layer_shell, ZwlrLayerShellV1 as LayerShell},
                    zwlr_layer_surface_v1 as layer_surface
                },
            };
            let layer_shell = env.require_global::<LayerShell>();
            let layer_surface = layer_shell.get_layer_surface(&surface, None, layer_shell::Layer::Overlay, "iced_sctk".to_string());
            //layer_surface.set_keyboard_interactivity(1);

            surface.commit();
            layer_surface.quick_assign({let surface = surface.clone(); move /*surface*/ |layer_surface, event, mut data| {
                let DispatchData::<A>{update: Update{streams, events, ..}, ..} = data.get().unwrap();
                use layer_surface::Event::*;
                match event {
                    Configure{serial, width, height} => {
                        if !(width > 0 && height > 0) {
                            layer_surface.set_size(size[0], size[1]);
                            layer_surface.ack_configure(serial);
                            surface.commit();
                            return;
                        }
                        layer_surface.ack_configure(serial);
                        events.push(Event::Window(iced_native::window::Event::Resized {width, height}));
                    }
                    Closed => quit(streams),
                    _ => unimplemented!(),
                }
            }});
            None
        } else {
            let window = env.create_window::<sctk::ConceptFrame, _>(surface.clone(), (settings.size[0], settings.size[1]),
                move |event, mut data| {
                    let DispatchData::<A>{update: Update{streams, events, .. }, state: State{window, ..}} = data.get().unwrap();
                    use sctk::Event::*;
                    match event {
                        Configure { new_size: None, .. } => (),
                        Configure { new_size: Some(new_size), .. } => {
                            window.size = [new_size.0, new_size.1];
                            events.push(Event::Window(iced_native::window::Event::Resized {width: new_size.0, height: new_size.1}));
                        }
                        Close => quit(streams),
                        Refresh => window.window.as_mut().unwrap().refresh(),
                    }
                }
            ).unwrap();
            window.set_resizable(settings.resizable);
            window.set_decorate(if settings.decorations { sctk::Decorations::FollowServer } else { sctk::Decorations::None });
            Some(window)
        };

        let (mut backend, renderer) = B::new(backend);

        struct Surface<'t>(&'t WlSurface);
        use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, unix::WaylandHandle};
        unsafe impl HasRawWindowHandle for Surface<'_> {
            fn raw_window_handle(&self) -> RawWindowHandle { RawWindowHandle::Wayland(WaylandHandle { /*TODO*/ ..WaylandHandle::empty() })  }
        }
        let surface = backend.create_surface(&Surface(&surface));
        let swap_chain = backend.create_swap_chain(&surface, size[0], size[1]);

        Self {
            window,
            size, scale_factor: 1,
            cursor: "left_ptr",
            title: Default::default(),
            mode: super::application::Mode::Windowed,
            backend, renderer, surface, swap_chain,
            buffer_size: [0,0], buffer_scale_factor: 0,
            cache: None,
        }
    }
    // After coalescing any size settings
    pub fn update_size(&mut self) -> bool {
        if self.buffer_size != self.size || self.buffer_scale_factor != self.scale_factor {
            //(self.buffer_size, self.buffer_scale_factor) = (self.size, self.scale_factor);
            self.buffer_size = self.size; self.buffer_scale_factor = self.scale_factor;
            self.swap_chain = self.backend.create_swap_chain(&self.surface,
                self.buffer_size[0] * self.buffer_scale_factor,
                self.buffer_size[1] * self.buffer_scale_factor);
            true
        } else {
            false
        }
    }
    pub fn update<A:crate::Application<Backend=B>>(&mut self, //runtime: &Runtime<Executor, Receiver, Message>, // fixme: trait Runtime
        application: &mut A, messages: Vec<A::Message>, events: Vec<Event>, trace: &mut Trace) -> &'static str {

        let _ = trace.scope(Layout);
        let mut user_interface = UserInterface::build(application.view(), self.size.into(), self.cache.take().unwrap_or_default(), &mut self.renderer);

        let messages = {
            // Deferred on_event(event, &mut messages) so user_interface mut borrows (application, renderer, debug)
            let mut sync_messages = user_interface.update(
                events,
                None, /*clipboard
                        .as_ref()
                        .map(|c| c as &dyn iced_native::Clipboard),*/
                &self.renderer,
            );
            sync_messages.extend(messages);
            sync_messages
        };

        let user_interface = {
            if !messages.is_empty() {
                let cache = user_interface.into_cache();
                // drop('user_interface &application)
                // yield messages;
                /*for message in messages {
                    log::debug!("Updating");
                    //debug.log_message(&message);
                    //debug.profile(Update);
                    runtime.spawn(runtime.enter(|| self.application.update(message)));
                }
                runtime.track(self.application.subscription());*/

                if let Some(window) = &self.window {
                    if self.title != application.title() {
                        self.title = application.title();
                        window.set_title(self.title.clone());
                    }
                    if self.mode != application.mode() {
                        self.mode = application.mode();
                        if let super::application::Mode::Fullscreen = self.mode { window.set_fullscreen(None) } else { window.unset_fullscreen() }
                    }
                }
                UserInterface::build(application.view(), self.size.into(), cache, &mut self.renderer)
            } else {
                user_interface
            }
        };

        let _ = trace.scope(Draw);
        let renderer_output = user_interface.draw(&mut self.renderer);
        self.cache = Some(user_interface.into_cache());
        let _ = trace.scope(Render);
        let cursor = self.backend.draw(&mut self.renderer, &mut self.swap_chain, &renderer_output, self.scale_factor as f64, &trace.lines());
        crate::conversion::cursor(cursor)
    }
}
