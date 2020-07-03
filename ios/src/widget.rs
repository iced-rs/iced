use crate::{
    //Layout,
    event::WidgetEvent,
    Hasher,
};
use uikit_sys::{
    UIView,
    id,
};
pub mod container;
pub mod checkbox;
pub mod text;
pub mod text_input;

pub use container::Container;
pub use checkbox::Checkbox;
pub use text::Text;
pub use text_input::TextInput;

pub struct WidgetPointers {
    pub root: id,
    pub others: Vec<id>,
    pub hash: u64,
}
impl Drop for WidgetPointers {
    fn drop(&mut self) {
        use uikit_sys::UIView_UIViewHierarchy;
        unsafe {
            let root = UIView(self.root);
            root.removeFromSuperview();
        }
    }
}

pub trait Widget<Message> {
    fn draw(
        &mut self,
        parent: UIView,
    ) -> WidgetPointers {
        WidgetPointers {
            root: parent.0,
            others: Vec::new(),
            hash: 0,
        }
    }
    fn hash_layout(&self, state: &mut Hasher);
    fn on_widget_event(
        &mut self,
        _event: WidgetEvent,
        //_layout: Layout<'_>,
        //_cursor_position: Point,
        _messages: &mut Vec<Message>,
        _widget_pointers: &WidgetPointers,
        //_renderer: &Renderer,
        //_clipboard: Option<&dyn Clipboard>,
    ) {
    }
}

#[allow(missing_debug_implementations)]
pub struct Element<'a, Message> {
    pub(crate) widget: Box<dyn Widget<Message> + 'a>,
}
impl<'a, Message> Element<'a, Message> {
    /// Create a new [`Element`] containing the given [`Widget`].
    ///
    /// [`Element`]: struct.Element.html
    /// [`Widget`]: widget/trait.Widget.html
    pub fn new(widget: impl Widget<Message> + 'a) -> Self {
        Self {
            widget: Box::new(widget),
        }
    }
}
impl<'a, Message> Widget<Message> for Element<'a, Message>
where
    Message: 'static,
{
    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);
        self.widget.hash_layout(state);
    }

}
