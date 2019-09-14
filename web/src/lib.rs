use futures::Future;

mod color;
mod element;
mod widget;

pub use color::Color;
pub use element::Element;
pub use iced::Align;
pub use widget::*;

pub trait UserInterface {
    type Message;

    fn update(
        &mut self,
        message: Self::Message,
    ) -> Box<dyn Future<Item = Self::Message, Error = ()>>;

    fn view(&mut self) -> Element<Self::Message>;

    fn run(mut self)
    where
        Self: Sized,
    {
        let element = self.view();
    }
}
