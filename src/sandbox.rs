use crate::{Application, Command, Element};

pub trait Sandbox {
    type Message: std::fmt::Debug + Send;

    fn new() -> Self;

    fn title(&self) -> String;

    fn update(&mut self, message: Self::Message);

    fn view(&mut self) -> Element<Self::Message>;

    fn run()
    where
        Self: 'static + Sized,
    {
        <Self as Application>::run()
    }
}

impl<T> Application for T
where
    T: Sandbox,
{
    type Message = T::Message;

    fn new() -> (Self, Command<T::Message>) {
        (T::new(), Command::none())
    }

    fn title(&self) -> String {
        T::title(self)
    }

    fn update(&mut self, message: T::Message) -> Command<T::Message> {
        T::update(self, message);

        Command::none()
    }

    fn view(&mut self) -> Element<T::Message> {
        T::view(self)
    }
}
