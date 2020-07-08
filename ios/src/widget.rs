use crate::{
    layout::{self,
        Layout,
    },
    Length,
    event::WidgetEvent,
    Hasher,
};
use uikit_sys::{
    UIView,
    id,
};

pub mod button;
pub mod checkbox;
pub mod container;
pub mod image;
pub mod progress_bar;
pub mod radio;
pub mod scrollable;
pub mod slider;
pub mod text_input;

mod column;
mod row;
mod space;
mod text;

#[doc(no_inline)]
pub use button::Button;
#[doc(no_inline)]
pub use scrollable::Scrollable;
#[doc(no_inline)]
pub use slider::Slider;
#[doc(no_inline)]
pub use text::Text;
#[doc(no_inline)]
pub use text_input::TextInput;

pub use checkbox::Checkbox;
pub use column::Column;
pub use container::Container;
pub use image::Image;
pub use progress_bar::ProgressBar;
pub use radio::Radio;
pub use row::Row;
pub use space::Space;

pub struct WidgetPointers {
    pub root: id,
    pub others: Vec<id>,
    pub hash: u64,
    //child: Option<Box<WidgetPointers>>,
}
/*
impl Drop for WidgetPointers {
    fn drop(&mut self) {
        use uikit_sys::UIView_UIViewHierarchy;
        unsafe {
            let root = UIView(self.root);
            root.removeFromSuperview();
        }
    }
}
*/

pub trait Widget<Message> {
    fn draw(
        &mut self,
        parent: UIView,
    ) -> WidgetPointers {
        WidgetPointers {
            root: parent.0,
            others: Vec::new(),
            hash: 0,
            //child: None
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
    fn layout(
        &self,
        limits: &layout::Limits,
    ) -> layout::Node;

    fn width(&self) -> Length;

    fn height(&self) -> Length;
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
{
    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);
        self.widget.hash_layout(state);
    }

    fn layout(
        &self,
        _limits: &layout::Limits,
    ) -> layout::Node {
        todo!();
    }

    fn width(&self) -> Length {
        todo!();
    }

    fn height(&self) -> Length {
        todo!();
    }

    fn draw(&mut self, parent: UIView) -> WidgetPointers {
        self.widget.draw(parent)
    }
    fn on_widget_event(
        &mut self,
        event: WidgetEvent,
        //_layout: Layout<'_>,
        //_cursor_position: Point,
        messages: &mut Vec<Message>,
        widget_pointers: &WidgetPointers,
        //_renderer: &Renderer,
        //_clipboard: Option<&dyn Clipboard>,
    ) {
        self.widget.on_widget_event(event, messages, widget_pointers);
    }
}
