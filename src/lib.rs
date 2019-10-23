#[cfg_attr(target_arch = "wasm32", path = "web.rs")]
#[cfg_attr(not(target_arch = "wasm32"), path = "winit.rs")]
mod platform;

pub use platform::*;

pub trait Application {
    type Message;

    fn update(&mut self, message: Self::Message);

    fn view(&mut self) -> Element<Self::Message>;

    fn run(self)
    where
        Self: 'static + Sized,
    {
        #[cfg(not(target_arch = "wasm32"))]
        iced_winit::Application::run(Instance(self));

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

    fn update(&mut self, message: Self::Message) {
        self.0.update(message);
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
