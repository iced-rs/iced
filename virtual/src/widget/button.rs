use crate::element::{self, Element};
use crate::Widget;

pub struct Button<Message, Renderer> {
    content: Element<Message, Renderer>,
    on_press: Option<Message>,
}

impl<Message, Renderer> Button<Message, Renderer> {
    pub fn new(
        content: impl element::Descriptor<Message, Renderer> + 'static,
    ) -> Self {
        Button {
            content: Element::new(content),
            on_press: None,
        }
    }

    pub fn on_press(mut self, on_press: Message) -> Self {
        self.on_press = Some(on_press);
        self
    }
}

impl<Message, Renderer> element::Descriptor<Message, Renderer>
    for Button<Message, Renderer>
where
    Message: 'static + Clone,
    Renderer: 'static,
{
    fn tag(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }

    fn build(&self) -> Box<dyn Widget<Message, Renderer>> {
        Box::new(State { is_pressed: false })
    }

    fn children(&self) -> &[Element<Message, Renderer>] {
        std::slice::from_ref(&self.content)
    }

    fn clone(&self) -> Box<dyn element::Descriptor<Message, Renderer>> {
        Box::new(Clone::clone(self))
    }
}

impl<Message, Renderer> Clone for Button<Message, Renderer>
where
    Message: Clone,
{
    fn clone(&self) -> Self {
        Self {
            content: self.content.clone(),
            on_press: self.on_press.clone(),
        }
    }
}

pub struct State {
    is_pressed: bool,
}

impl<Message, Renderer> Widget<Message, Renderer> for State {}
