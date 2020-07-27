use crate::{event::WidgetEvent, Hasher, Length};
use std::cell::RefCell;
use std::rc::Rc;
use uikit_sys::{id, UIView};

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

#[derive(Debug, Clone)]
pub struct WidgetNode {
    pub(crate) view_id: id,
    pub(crate) hash: u64,
    //pub (crate) widget_id: u64,

    // Used for memory collection.
    related_ids: Vec<id>,
    pub widget_type: WidgetType,
    // Used in things like Row, Column and Container.
    //pub(crate) children: Vec<Rc<RefCell<WidgetNode>>>,
}

impl PartialEq for WidgetNode {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash && self.widget_type == other.widget_type
    }
}

impl Default for WidgetNode {
    fn default() -> Self {
        Self {
            view_id: 0 as id,
            hash: 0,
            related_ids: Vec::new(),
            widget_type: WidgetType::BaseElement,
            //children: Vec::new(),
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
    pub fn new(view_id: id, widget_type: WidgetType, hash: u64) -> Self {
        Self {
            view_id,
            hash,
            related_ids: Vec::new(),
            widget_type,
            //children: Vec::new(),
        }
    }

    pub fn drop_from_ui(&self) {
        debug!("DROPPING A WIDGET NODE! {:?}", self.view_id);
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
        use WidgetType::*;
        match (&self.widget_type, &other.widget_type) {
            (Text(_), Text(_))
            | (Column(_), Column(_))
            | (TextInput, TextInput) => true,
            _ => false,
        }
    }

    pub fn merge(&mut self, other: &Self, root_view: Option<UIView>) {
        use std::convert::TryInto;
        use uikit_sys::{IUIStackView, UIStackView, UIView_UIViewHierarchy};

        if !self.is_mergeable(other) {
            let old_self = self.clone();
            *self = Self { ..other.clone() };
            if let Some(parent) = root_view {
                self.draw(parent);
                old_self.drop_from_ui();
                /*

                let old_view = UIView(self.view_id);
                unsafe {
                    old_view.removeFromSuperview();
                }
                */
            }
            old_self.drop_from_ui();
        } else {
            match (&mut self.widget_type, &other.widget_type) {
                (
                    WidgetType::Column(ref mut my_children),
                    WidgetType::Column(other_children),
                ) => {
                    let stackview = UIStackView(self.view_id);
                    if my_children.len() == other_children.len() {
                        for i in 0..my_children.len() {
                            let current_child = my_children.get_mut(i).unwrap();
                            let new_child = other_children.get(i).unwrap();

                            if current_child
                                .borrow()
                                .is_mergeable(&new_child.borrow())
                            {
                                current_child
                                    .borrow_mut()
                                    .merge(&new_child.borrow(), None);
                            } else {
                                unsafe {
                                    stackview.removeArrangedSubview_(
                                        UIView(current_child.borrow().view_id)
                                    );
                                    current_child.borrow().drop_from_ui();
                                    stackview.insertArrangedSubview_atIndex_(
                                        UIView(new_child.borrow().view_id),
                                        i.try_into().unwrap(),
                                    );
                                }
                                my_children[i] = new_child.clone();
                            }
                        }
                    } else {
                        let stackview = uikit_sys::UIStackView(self.view_id);
                        for i in my_children.clone() {
                            unsafe {
                                stackview
                                    .removeArrangedSubview_(UIView(i.borrow().view_id))
                            }
                            i.borrow().drop_from_ui();
                        }
                        *my_children = other_children.clone();
                        for i in my_children {
                            unsafe {
                                stackview
                                    .addArrangedSubview_(UIView(i.borrow().view_id))
                            }
                        }
                    }
                }
                (
                    WidgetType::Text(current_text),
                    WidgetType::Text(new_text),
                ) => {
                    debug!(
                        "Updating text from {} to {}",
                        current_text, new_text
                    );
                    use std::convert::TryInto;
                    use std::ffi::CString;
                    use uikit_sys::{
                        IUITextView, NSString,
                        NSString_NSStringExtensionMethods,
                        NSUTF8StringEncoding, UITextView,
                    };
                    let label = UITextView(self.view_id);
                    unsafe {
                        let text = NSString(
                            NSString::alloc().initWithBytes_length_encoding_(
                                CString::new(new_text.as_str())
                                    .expect("CString::new failed")
                                    .as_ptr()
                                    as *mut std::ffi::c_void,
                                new_text.len().try_into().unwrap(),
                                NSUTF8StringEncoding,
                            ),
                        );
                        label.setText_(text);
                    }
                }
                (
                    WidgetType::TextInput,
                    WidgetType::TextInput,
                ) => {
                    debug!("Updating text input widgets is not implemented yet");
                    //TODO: Add stuff about Text Input
                }
                (me, you) => {
                    debug!("Widget's don't match! {:?}, {:?}", me, you);
                }
            }
        }
    }

    pub fn add_related_id(&mut self, related_id: id) {
        self.related_ids.push(related_id);
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

#[derive(Debug)]
pub enum RenderAction {
    Add,
    Remove,
    Update,
}

pub trait Widget<Message> {
    fn get_widget_type(&self) -> WidgetType;

    fn get_widget_node(&self) -> WidgetNode {
        let hash = self.get_my_hash();
        WidgetNode::new(0 as id, self.get_widget_type(), hash)
    }

    fn build_uiview(&self, is_root: bool) -> WidgetNode {
        unimplemented!(
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
        //_layout: Layout<'_>,
        //_cursor_position: Point,
        _messages: &mut Vec<Message>,
        _widget_node: &WidgetNode,
        //_renderer: &Renderer,
        //_clipboard: Option<&dyn Clipboard>,
    ) {
        debug!("on_widget_event for {:?}", self.get_widget_type());
    }

    fn get_render_action(
        &self,
        widget_node: Option<&WidgetNode>,
    ) -> RenderAction {
        let action = if widget_node.is_none() {
            RenderAction::Add
        } else if widget_node.is_some()
            && widget_node.unwrap().widget_type == self.get_widget_type()
        {
            RenderAction::Update
        } else {
            RenderAction::Remove
        };
        debug!(
            "RENDER ACTION FOR WIDGET {:?} is {:?}",
            self.get_widget_type(),
            action
        );
        action
    }

    fn width(&self) -> Length;

    fn height(&self) -> Length;
}

//pub type Element<'a, Message> = ElementTemplate<Message, Box<dyn Widget<Message> + 'a>>;

#[allow(missing_debug_implementations)]
pub struct Element<'a, Message> {
    pub widget: Box<dyn Widget<Message> + 'a>,
}

/*
impl<T: Widget<Message>> Into<WidgetNode> for T {
    fn into(self) -> WidgetNode {
        let mut node = WidgetNode::new(None, self.get_widget_type());
        for i in &self.get_element_children() {
            node.add_child(i.widget.into());
        }
        node
    }
}
*/

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

    fn width(&self) -> Length {
        self.widget.width()
    }

    fn height(&self) -> Length {
        self.widget.height()
    }

    fn get_widget_type(&self) -> WidgetType {
        self.widget.get_widget_type()
    }
    fn get_render_action(
        &self,
        widget_node: Option<&WidgetNode>,
    ) -> RenderAction {
        self.widget.get_render_action(widget_node)
    }

    fn build_uiview(&self, is_root: bool) -> WidgetNode {
        self.widget.build_uiview(is_root)
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
impl<'a, Message> From<Element<'a, Message>> for WidgetNode {
    fn from(element: Element<'a, Message>) -> WidgetNode {
        Widget::from(element.widget)
    }
}
impl<'a, Message> From<Box<dyn Widget<Message> + 'a>> for WidgetNode {
    fn from(element: Box<dyn Widget<Message> + 'a>) -> WidgetNode {
        Widget::from(element)
    }
}
impl<'a, Message> From<&Element<'a, Message>> for WidgetNode {
    fn from(element: &Element<'a, Message>) -> WidgetNode {
        Widget::from(element.widget)
    }
}
*/
