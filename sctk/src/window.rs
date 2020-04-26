use smithay_client_toolkit::window::{Window as SCTKWindow, ConceptFrame, Decorations};
use iced_native::{Runtime, Trace, UserInterface, Cache, window, Event};
use super::pointer::Pointer;

type Cursor = &'static str;

pub enum Mode { Windowed, Fullscreen }

pub struct Settings {
    pub size: (u32, u32),
    pub resizable: bool,
    pub decorations: bool,
    pub overlay: bool,
}
impl Default for Settings { fn default() -> Self { Self{ resizable: true, decorations: true, ..Default::default() } } }

pub struct Window<Backend:window::Backend, Renderer:iced_native::Renderer> {
    pub window: SCTKWindow<ConceptFrame>, // Refresh window
    pub size: (u32, u32), pub scale_factor: u32, // Configure window
    pub current_cursor: Cursor, // pointer::Enter window
    pub pointer: Option<Pointer>, // render set_pointer

    title: String,
    mode: Mode,
    backend : Backend, renderer : Renderer, swap_chain: Backend::SwapChain,
    pub buffer_size: (u32, u32), pub buffer_scale_factor: u32, // fixme: should be in swap_chain
    cache: Option<Cache>
}

impl<Backend:window::Backend, Renderer:iced_native::Renderer> Window<Backend, Renderer> {
    fn new(window: SCTKWindow<ConceptFrame>, settings: Settings, backend: Backend::Settings) -> Self {
        window.set_resizable(settings.resizable);
        window.set_decorate(
            if settings.decorations { Decorations::FollowServer }
            else { Decorations::None }
        );

        let size = settings.size;
        let (mut backend, mut renderer) = Backend::new(backend);
        let mut swap_chain = backend.create_swap_chain(&backend.create_surface(&window.surface), size.0, size.1);

        Self {
            window,
            size, scale_factor: 1,
            current_cursor: "left_ptr",
            backend, renderer, swap_chain,
            cache: None,
        }
    }
    fn update<Executor, Receiver, Message>(&mut self, runtime: &Runtime<Executor, Receiver, Message>, // fixme: trait Runtime
                                                                                                                          events: Vec<Event>, messages: Vec<Message>) -> Renderer::Output {
        //debug.profile(Layout);
        let mut user_interface = UserInterface::build(self.application.view(), self.size.into(), self.cache.unwrap_or(Cache::new()), self.renderer);

        let messages = {
            // Deferred on_event(event, &mut messages) so user_interface mut borrows (application, renderer, debug)
            let mut sync_messages = user_interface.update(
                events.drain(..),
                None, /*clipboard
                        .as_ref()
                        .map(|c| c as &dyn iced_native::Clipboard),*/
                &self.renderer,
            );
            sync_messages.extend(messages);
            sync_messages
        };

        let user_interface = {
            if messages.len() > 0 {
                let cache = user_interface.into_cache();
                // drop('user_interface &application)
                // yield messages;
                for message in messages {
                    log::debug!("Updating");
                    //debug.log_message(&message);
                    //debug.profile(Update);
                    runtime.spawn(runtime.enter(|| self.application.update(message)));
                }
                runtime.track(self.application.subscription());

                // fixme
                if self.title != self.application.title() {
                    self.title = self.application.title();
                    self.window.set_title(self.title.clone());
                }
                if self.mode != self.application.mode() {
                    self.mode = self.application.mode();
                    if let Mode::Fullscreen = self.mode { self.window.set_fullscreen(None) } else { self.window.unset_fullscreen() }
                }

                UserInterface::build(self.application.view(), self.size.into(), cache.unwrap_or(Cache::new()), self.renderer);
            } else {
                user_interface
            }
        };

        //debug.profile(Draw);
        let renderer_output = user_interface.draw(&mut self.renderer);
        self.cache = Some(user_interface.into_cache());
        renderer_output
    }
    fn render(&mut self, renderer_output: Renderer::Output, trace: &Trace) -> Cursor {
        //debug.profile(Render);

        let cursor = self.backend.draw(&mut self.renderer, &mut self.swap_chain, &renderer_output, self.scale_factor as f64, &trace.overlay());

        if self.cursor != cursor {
            self.cursor = cursor;
            for pointer in self.pointer.iter_mut() {
                pointer.set_cursor(crate::conversion::cursor(cursor), None).expect("Unknown cursor");
            }
        }
    }
}
