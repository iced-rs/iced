use {iced_native::{Executor, Command, Element, Subscription}, super::async_sctk};

/// The mode of a window-based application.
#[derive(PartialEq, Debug)] pub enum Mode {
    /// The application appears in its own window.
    Windowed,
    /// The application takes the whole screen of its current monitor.
    Fullscreen
}

// The graphics backend either software rendering to shared memory (iced_shm) or WGPU (iced_wgpu)
cfg_if::cfg_if! { if #[cfg(feature="wayland-client/use_system_lib")] { pub use iced_shm::window::ShmBackend as Backend; }
                        else { pub trait Backend = iced_shm::window::ShmBackend<Surface=smithay_client_toolkit::reexports::client::protocol::wl_surface::WlSurface>; } }

///
#[derive(Debug)]
pub struct Settings<Flags> {
    /// Arguments forwarded from 'run' to 'new' inside Runtime
    pub flags: Flags,
    /// Window settings
    pub window: async_sctk::Settings,
}

/// An SCTK Application (compatible with winit backend except Backend->ShmBackend) (FIXME: share high level interface specification)
pub trait Application: Sized {
    /// The graphics backend either software rendering to shared memory (iced_shm) or WGPU (iced_wgpu)
    type Backend: Backend<Surface=smithay_client_toolkit::reexports::client::protocol::wl_surface::WlSurface>; //Backend;
    /// Run commands and subscriptions
    type Executor: Executor;
    /// User-specific application messages
    type Message: std::fmt::Debug + Send;
    /// Arguments forwarded from 'run' to 'new' inside Runtime
    type Flags;
    /// Initial state from 'run' arguments. Executed inside Runtime
    fn new(arguments: Self::Flags) -> (Self, Command<Self::Message>);
    /// Title evaluated after update
    fn title(&self) -> String;
    /// Updates application state from message
    fn update(&mut self, message: Self::Message) -> Command<Self::Message>;
    /// Subscription evaluated after update
    fn subscription(&self) -> Subscription<Self::Message>;
    /// Displayed widget tree evaluated after update
    fn view(&mut self) -> Element<'_, Self::Message, <Self::Backend as /*Backend*/iced_shm::window::ShmBackend>::Renderer>;
    /// Windowed/Fullscreen mode evaluated after update
    fn mode(&self) -> Mode { Mode::Windowed }
    /// Blocking application event loop
    fn run(settings: Settings<Self::Flags>, backend: <Self::Backend as /*Backend*/iced_shm::window::ShmBackend>::Settings) where Self:'static {
        smol::run(async_sctk::application::<Self>(settings.flags, settings.window, backend).unwrap()).unwrap()
    }
}
