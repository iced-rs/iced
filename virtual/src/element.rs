use crate::Widget;

pub struct Element<Message, Renderer> {
    widget: Box<dyn Descriptor<Message, Renderer>>,
}

impl<Message, Renderer> Element<Message, Renderer> {
    pub fn new(widget: impl Descriptor<Message, Renderer> + 'static) -> Self {
        Self {
            widget: Box::new(widget),
        }
    }
}

pub trait Descriptor<Message, Renderer> {
    fn tag(&self) -> std::any::TypeId;

    fn build(&self) -> Box<dyn Widget<Message, Renderer>>;

    fn children(&self) -> &[Element<Message, Renderer>];

    fn clone(&self) -> Box<dyn Descriptor<Message, Renderer>>;
}

impl<Message, Renderer> Clone for Box<dyn Descriptor<Message, Renderer>> {
    fn clone(&self) -> Self {
        self.as_ref().clone()
    }
}

impl<Message, Renderer> Clone for Element<Message, Renderer> {
    fn clone(&self) -> Self {
        Element {
            widget: self.widget.clone(),
        }
    }
}
