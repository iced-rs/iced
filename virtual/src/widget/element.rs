use crate::Widget;

pub struct Element<Message, Renderer> {
    widget: Box<dyn Widget<Message, Renderer>>,
}

impl<Message, Renderer> Element<Message, Renderer> {
    pub fn new(widget: impl Widget<Message, Renderer> + 'static) -> Self {
        Self {
            widget: Box::new(widget),
        }
    }

    pub fn as_widget(&self) -> &dyn Widget<Message, Renderer> {
        self.widget.as_ref()
    }

    pub fn as_widget_mut(&mut self) -> &mut dyn Widget<Message, Renderer> {
        self.widget.as_mut()
    }
}
