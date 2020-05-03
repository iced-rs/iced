use {iced_native::{window, Executor, Command, Element, Subscription}, super::{Settings, async_sctk}};

/// The mode of a window-based application.
#[derive(PartialEq, Debug)] pub enum Mode {
    /// The application appears in its own window.
    Windowed,
    /// The application takes the whole screen of its current monitor.
    Fullscreen
}

/// An SCTK Application (compatible with winit backend) (FIXME: share high level interface specification)
pub trait Application: Sized {
    /// The graphics backend either software rendering to shared memory (iced_shm) or WGPU (iced_wgpu)
    type Backend: window::Backend;// + 'static; // wayland-client/DispatchData:Any:'static
    /// Run commands and subscriptions
    type Executor: Executor;
    /// User-specific application messages
    type Message: std::fmt::Debug + Send;// + 'static; // wayland-client/DispatchData:Any:'static
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
    fn view(&mut self) -> Element<'_, Self::Message, <Self::Backend as window::Backend>::Renderer>;
    /// Windowed/Fullscreen mode evaluated after update
    fn mode(&self) -> Mode { Mode::Windowed }
    /// Blocking application event loop
    fn run(settings: Settings<Self::Flags>, backend: <Self::Backend as window::Backend>::Settings) where Self:'static {
        smol::run(async_sctk::application::<Self>(settings.flags, settings.window, backend).unwrap()).unwrap()
    }
}
