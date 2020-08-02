use crate::{event::WidgetEvent, Hasher, Length};
use std::cell::RefCell;
use std::rc::Rc;
use uikit_sys::{id, UIView};
use std::convert::TryInto;

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
    Text(String),
    TextInput,
    Checkbox,
    Column(Vec<Rc<RefCell<WidgetNode>>>),
    Container,
    Image,
    ProgressBar,
    Radio,
    Row,
    Space,
}

impl WidgetType {
    fn is_mergeable(&self, other: &Self) -> bool {
        use WidgetType::*;
        match (&self, &other) {
            (Text(_), Text(_))
                | (Column(_), Column(_))
                | (TextInput, TextInput) => true,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct WidgetNode {
    pub(crate) view_id: id,
    pub(crate) hash: u64,

    // Used for memory collection.
    related_ids: Vec<id>,
    pub widget_type: WidgetType,
}

use std::fmt;
impl fmt::Debug for WidgetNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WidgetNode")
            .field("view_id", &self.view_id)
            .field("hash", &self.hash)
            .field("widget_type", &self.widget_type)
            .finish()
    }
}


impl PartialEq for WidgetNode {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash && self.widget_type == other.widget_type
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
    pub fn new(view_id: id, widget_type: WidgetType, hash: u64) -> Self {
        Self {
            view_id,
            hash,
            related_ids: Vec::new(),
            widget_type,
        }
    }

    pub fn drop_from_ui(&self) {
        trace!("DROPPING A WIDGET NODE! {:?}", self.view_id);
        use uikit_sys::{INSObject, NSObject, UIView_UIViewHierarchy};
        if self.view_id != 0 as id {
            let view = UIView(self.view_id);
            unsafe {
                view.removeFromSuperview();
                //view.dealloc();
            }
        }
        for i in &self.related_ids {
            let obj = NSObject(*i);
            unsafe {
                obj.dealloc();
            }
        }
        match &self.widget_type {
            WidgetType::Column(ref children) => {
                for i in children {
                    i.borrow().drop_from_ui();
                }
            }
            _ => {}
        }
    }

    pub fn draw(&self, parent: UIView) {
        use uikit_sys::UIView_UIViewHierarchy;
        if self.view_id != 0 as id {
            unsafe {
                parent.addSubview_(UIView(self.view_id));
            }
        }
    }

    fn is_mergeable(&self, other: &Self) -> bool {
        self.widget_type.is_mergeable(&other.widget_type)
    }

    pub fn add_related_id(&mut self, related_id: id) {
        self.related_ids.push(related_id);
    }

    fn replace_child(&mut self, child: WidgetNode, i: usize) {
        match &mut self.widget_type {
            WidgetType::Column(ref mut children) => {
                if i < children.len() {
                    children[i] = Rc::new(RefCell::new(child));
                }
            }
            e => {
                unimplemented!("REPLACE CHILD IS NOT IMPLEMENTED FOR {:?}", e);
            }
        }

    }
    pub fn drop_children(&mut self) {
        match &mut self.widget_type {
            WidgetType::Column(ref mut children) => {
                *children = Vec::new();
            }
            e => {
                unimplemented!("DROP CHILDREN ARE NOT IMPLEMENTED FOR {:?}", e);
            }
        }
    }

    pub fn add_child(&mut self, child: WidgetNode) {
        match &mut self.widget_type {
            WidgetType::Column(ref mut children) => {
                children.push(Rc::new(RefCell::new(child)));
            }
            e => {
                unimplemented!("CHILDREN ARE NOT IMPLEMENTED FOR {:?}", e);
            }
        }
    }
    pub fn add_children(&mut self, _children: Vec<WidgetNode>) {
        /*
        for i in &children {
            self.add_child(*i);
        }
        */
    }
}

pub trait Widget<Message> {
    fn get_widget_type(&self) -> WidgetType;

    fn build_uiview(&self, _is_root: bool) -> WidgetNode {
        unimplemented!(
            "{:?} using base implementation",
            self.get_widget_type()
        );
    }

    fn update(&self, current_node: &mut WidgetNode, root_view: Option<UIView>) {
        error!(
            "{:?} using base implementation",
            self.get_widget_type()
        );
    }

    fn get_my_hash(&self) -> u64 {
        use std::hash::Hasher;
        let hasher = &mut crate::Hasher::default();
        self.hash_layout(hasher);

        hasher.finish()
    }

    fn hash_layout(&self, state: &mut Hasher);
    fn on_widget_event(
        &mut self,
        _event: WidgetEvent,
        _messages: &mut Vec<Message>,
        _widget_node: &WidgetNode,
    ) {
        trace!("on_widget_event for {:?}", self.get_widget_type());
    }

    fn width(&self) -> Length;

    fn height(&self) -> Length;
}


#[allow(missing_debug_implementations)]
pub struct Element<'a, Message> {
    pub widget: Box<dyn Widget<Message> + 'a>,
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

impl<'a, Message> Widget<Message> for Element<'a, Message> {
    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);
        self.widget.hash_layout(state);
    }

    fn update(&self, current_node: &mut WidgetNode, root_view: Option<UIView>) {
        self.widget.update(current_node, root_view)
    }


    fn width(&self) -> Length {
        self.widget.width()
    }

    fn height(&self) -> Length {
        self.widget.height()
    }

    fn get_widget_type(&self) -> WidgetType {
        self.widget.get_widget_type()
    }

    fn build_uiview(&self, is_root: bool) -> WidgetNode {
        self.widget.build_uiview(is_root)
    }

    fn on_widget_event(
        &mut self,
        event: WidgetEvent,
        messages: &mut Vec<Message>,
        widget_node: &WidgetNode,
    ) {
        self.widget.on_widget_event(event, messages, widget_node);
    }
}
