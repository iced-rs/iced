#[cfg_attr(target_arch = "wasm32", path = "web.rs")]
#[cfg_attr(not(target_arch = "wasm32"), path = "winit.rs")]
mod platform;

pub use platform::*;

pub trait Application {
    type Message: std::fmt::Debug;

    fn title(&self) -> String;

    fn update(&mut self, message: Self::Message);

    fn view(&mut self) -> Element<Self::Message>;

    fn new(self) -> (platform::EventLoop<()>, platform::Window)
    where
        Self: 'static + Sized,
    {
        #[cfg(not(target_arch = "wasm32"))]
        return iced_winit::Application::new(Instance(self));

        #[cfg(target_arch = "wasm32")]
        ((), ())
    }

    fn run(self, event_loop : platform::EventLoop<()>, window : platform::Window)
    where
        Self: 'static + Sized,
    {
        #[cfg(not(target_arch = "wasm32"))]
        iced_winit::Application::run(Instance(self), event_loop, window);

        #[cfg(target_arch = "wasm32")]
        iced_web::Application::new_run(Instance(self));
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

    fn title(&self) -> String {
        self.0.title()
    }

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
