#[cfg_attr(target_arch = "wasm32", path = "web.rs")]
#[cfg_attr(not(target_arch = "wasm32"), path = "winit.rs")]
mod platform;

pub use platform::*;

pub trait Application: Sized {
    type Message: std::fmt::Debug + Send;

    fn new() -> (Self, Command<Self::Message>);

    fn title(&self) -> String;

    fn update(&mut self, message: Self::Message) -> Command<Self::Message>;

    fn view(&mut self) -> Element<Self::Message>;

    fn run()
    where
        Self: 'static + Sized,
    {
        #[cfg(not(target_arch = "wasm32"))]
        <Instance<Self> as iced_winit::Application>::run();

        #[cfg(target_arch = "wasm32")]
        iced_web::Application::run(Instance(self));
    }
}

struct Instance<A: Application>(A);

#[cfg(not(target_arch = "wasm32"))]
impl<A> iced_winit::Application for Instance<A>
where
    A: Application,
{
    type Renderer = Renderer;
    type Message = A::Message;

    fn new() -> (Self, Command<A::Message>) {
        let (app, command) = A::new();

        (Instance(app), command)
    }

    fn title(&self) -> String {
        self.0.title()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        self.0.update(message)
    }

    fn view(&mut self) -> Element<Self::Message> {
        self.0.view()
    }
}

#[cfg(target_arch = "wasm32")]
impl<A> iced_web::Application for Instance<A>
where
    A: Application,
{
    type Message = A::Message;

    fn update(&mut self, message: Self::Message) {
        self.0.update(message);
    }

    fn view(&mut self) -> Element<Self::Message> {
        self.0.view()
    }
}
