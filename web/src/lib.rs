use dodrio::bumpalo;
use std::cell::RefCell;

mod bus;
mod element;
pub mod widget;

pub use bus::Bus;
pub use element::Element;
pub use iced_core::{
    Align, Background, Color, Font, HorizontalAlignment, Length,
    VerticalAlignment,
};
pub use widget::*;

pub trait Application {
    type Message;

    fn update(&mut self, message: Self::Message);

    fn view(&mut self) -> Element<Self::Message>;

    fn run(self)
    where
        Self: 'static + Sized,
    {
        let app = Instance::new(self);

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();
        let vdom = dodrio::Vdom::new(&body, app);

        vdom.forget();
    }
}

struct Instance<Message> {
    ui: RefCell<Box<dyn Application<Message = Message>>>,
}

impl<Message> Instance<Message> {
    fn new(ui: impl Application<Message = Message> + 'static) -> Self {
        Self {
            ui: RefCell::new(Box::new(ui)),
        }
    }

    fn update(&mut self, message: Message) {
        self.ui.borrow_mut().update(message);
    }
}

impl<Message> dodrio::Render for Instance<Message>
where
    Message: 'static,
{
    fn render<'a, 'bump>(
        &'a self,
        bump: &'bump bumpalo::Bump,
    ) -> dodrio::Node<'bump>
    where
        'a: 'bump,
    {
        let mut ui = self.ui.borrow_mut();
        let element = ui.view();

        element.widget.node(bump, &Bus::new())
    }
}
