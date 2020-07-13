use crate::{
    Length,
    event::WidgetEvent,
    Hasher,
};
use uikit_sys::{
    UIView,
    id,
};

/*
pub mod button;
pub mod checkbox;
pub mod container;
pub mod image;
pub mod progress_bar;
pub mod radio;
pub mod scrollable;
pub mod slider;
mod row;
mod space;
*/
pub mod text_input;

mod column;
mod text;

/*
#[doc(no_inline)]
pub use button::Button;
#[doc(no_inline)]
pub use scrollable::Scrollable;
#[doc(no_inline)]
pub use slider::Slider;
*/
#[doc(no_inline)]
pub use text::Text;
#[doc(no_inline)]
pub use text_input::TextInput;

pub use column::Column;
/*
pub use checkbox::Checkbox;
pub use container::Container;
pub use image::Image;
pub use progress_bar::ProgressBar;
pub use radio::Radio;
pub use row::Row;
pub use space::Space;
*/

#[derive(Debug, Clone, PartialEq)]
pub enum WidgetType {
    BaseElement,
    Button,
    Scrollable,
    Slider,
    Text,
    TextInput,
    Checkbox,
    Column,
    Container,
    Image,
    ProgressBar,
    Radio,
    Row,
    Space,
}
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct WidgetNode {
    pub (crate) view_id: Option<id>,
    //pub (crate) widget_id: u64,

    // Used for memory collection.
    related_ids: Vec<id>,
    pub widget_type: WidgetType,
    // Used in things like Row, Column and Container.
    pub (crate) children: Vec<Rc<RefCell<WidgetNode>>>,
}

impl Default for WidgetNode {
    fn default() -> Self {
        Self {
            view_id: None,
            //widget_id: 0,
            related_ids: Vec::new(),
            widget_type: WidgetType::BaseElement,
            children: Vec::new(),
        }
    }
}

/*
impl Drop for WidgetNode {
    fn drop(&mut self) {
        debug!("DROPPING A WIDGET NODE! {:?}", self.view_id);
        use uikit_sys::{
            UIView_UIViewHierarchy,
            NSObject, INSObject,
        };
        unsafe {
            let view = UIView(self.view_id);
            view.removeFromSuperview();
            view.dealloc();
        }
        for i in &self.related_ids {
            let obj = NSObject(*i);
            unsafe {
                obj.dealloc();
            }
        }
        for i in &self.children {
            drop(i);
        }
    }
}
*/
impl WidgetNode {
    pub fn new(view_id: Option<id>, widget_type: WidgetType) -> Self {
        Self {
            view_id,
            related_ids: Vec::new(),
            widget_type,
            children: Vec::new(),
        }
    }

    pub fn drop_from_ui(&self) {
        debug!("DROPPING A WIDGET NODE! {:?}", self.view_id);
        use uikit_sys::{
            UIView_UIViewHierarchy,
            NSObject, INSObject,
        };
        if let Some(view_id) = self.view_id {
            let view = UIView(view_id);
            unsafe {
                view.removeFromSuperview();
                view.dealloc();
            }
        }
        for i in &self.related_ids {
            let obj = NSObject(*i);
            unsafe {
                obj.dealloc();
            }
        }
        for i in &self.children {
            i.borrow().drop_from_ui();
        }
    }

    pub fn add_related_id(&mut self, related_id: id) {
        self.related_ids.push(related_id);
    }

    pub fn add_child(&mut self, child: WidgetNode) {
        self.children.push(Rc::new(RefCell::new(child)));
    }
    pub fn add_children(&mut self, _children: Vec<WidgetNode>) {
        /*
        for i in &children {
            self.add_child(*i);
        }
        */
    }
}

#[derive(Debug)]
pub enum RenderAction {
    Add,
    Remove,
    Update,
}

pub trait Widget<Message> //: Into<WidgetNode> + Sized
{
    fn update_or_add(
        &mut self,
        _parent: Option<UIView>,
        _old_node: Option<WidgetNode>,
    ) -> WidgetNode {
        unimplemented!("USING BASE IMPLEMENTATION FOR UPDATE_OR_ADD");
    }
    fn get_widget_type(&self) -> WidgetType {
        unimplemented!("USING BASE IMPLEMENTATION for GET_WIDGET_TYPE");
    }
    fn hash_layout(&self, state: &mut Hasher);
    fn on_widget_event(
        &mut self,
        _event: WidgetEvent,
        //_layout: Layout<'_>,
        //_cursor_position: Point,
        _messages: &mut Vec<Message>,
        _widget_node: &WidgetNode,
        //_renderer: &Renderer,
        //_clipboard: Option<&dyn Clipboard>,
    ) {
        debug!("on_widget_event for {:?}", self.get_widget_type());
    }

    fn get_render_action(&self, widget_node: Option<&WidgetNode>) -> RenderAction {
        let action = if widget_node.is_none() {
            RenderAction::Add
        } else if widget_node.is_some() && widget_node.unwrap().widget_type == self.get_widget_type() {
            RenderAction::Update
        } else {
            RenderAction::Remove
        };
        debug!("RENDER ACTION FOR WIDGET {:?} is {:?}", self.get_widget_type(), action);
        action
    }

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

    fn width(&self) -> Length {
        self.widget.width()
    }

    fn height(&self) -> Length {
        self.widget.height()
    }

    fn update_or_add(&mut self, parent: Option<UIView>, old_node: Option<WidgetNode>) -> WidgetNode {
        self.widget.update_or_add(parent, old_node)
    }

    fn get_widget_type(&self) -> WidgetType {
        self.widget.get_widget_type()
    }
    fn get_render_action(&self, widget_node: Option<&WidgetNode>) -> RenderAction {
        self.widget.get_render_action(widget_node)
    }

    fn on_widget_event(
        &mut self,
        event: WidgetEvent,
        //_layout: Layout<'_>,
        //_cursor_position: Point,
        messages: &mut Vec<Message>,
        widget_node: &WidgetNode,
        //_renderer: &Renderer,
        //_clipboard: Option<&dyn Clipboard>,
    ) {
        self.widget.on_widget_event(event, messages, widget_node);
    }
}

/*
impl<'a, Message> From<Box<dyn Widget<Message> + 'a>> for WidgetNode {
    fn from(element: Box<dyn Widget<Message> + 'a>) -> WidgetNode {
        Widget::from(element)
    }
}
impl<'a, Message> From<Element<'a, Message>> for WidgetNode {
    fn from(element: Element<'a, Message>) -> WidgetNode {
        Widget::from(element.widget)
    }
}
impl<'a, Message> From<&Element<'a, Message>> for WidgetNode {
    fn from(element: &Element<'a, Message>) -> WidgetNode {
        Widget::from(element.widget)
    }
}
*/
