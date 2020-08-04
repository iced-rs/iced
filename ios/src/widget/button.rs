//! Allow your users to perform actions by pressing a button.
//!
//! A [`Button`] has some local [`State`].
//!
//! [`Button`]: struct.Button.html
//! [`State`]: struct.State.html
use crate::{
    widget::{WidgetNode, WidgetType},
    event::{EventHandler, WidgetEvent},
    Background, Element, Hasher, Length, Point, Widget,
};
use std::hash::Hash;

pub use iced_style::button::{Style, StyleSheet};

/// A generic widget that produces a message when pressed.
///
/// ```
/// # use iced_web::{button, Button, Text};
/// #
/// enum Message {
///     ButtonPressed,
/// }
///
/// let mut state = button::State::new();
/// let button = Button::new(&mut state, Text::new("Press me!"))
///     .on_press(Message::ButtonPressed);
/// ```
#[allow(missing_debug_implementations)]
pub struct Button<'a, Message> {
    content: Element<'a, Message>,
    on_press: Option<Message>,
    width: Length,
    height: Length,
    min_width: u32,
    min_height: u32,
    padding: u16,
    style: Box<dyn StyleSheet>,
}

impl<'a, Message> Button<'a, Message> {
    /// Creates a new [`Button`] with some local [`State`] and the given
    /// content.
    ///
    /// [`Button`]: struct.Button.html
    /// [`State`]: struct.State.html
    pub fn new<E>(_state: &'a mut State, content: E) -> Self
    where
        E: Into<Element<'a, Message>>,
    {
        Button {
            content: content.into(),
            on_press: None,
            width: Length::Shrink,
            height: Length::Shrink,
            min_width: 0,
            min_height: 0,
            padding: 5,
            style: Default::default(),
        }
    }

    /// Sets the width of the [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the minimum width of the [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    pub fn min_width(mut self, min_width: u32) -> Self {
        self.min_width = min_width;
        self
    }

    /// Sets the minimum height of the [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    pub fn min_height(mut self, min_height: u32) -> Self {
        self.min_height = min_height;
        self
    }

    /// Sets the padding of the [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the style of the [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    pub fn style(mut self, style: impl Into<Box<dyn StyleSheet>>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed.
    ///
    /// [`Button`]: struct.Button.html
    pub fn on_press(mut self, msg: Message) -> Self {
        self.on_press = Some(msg);
        self
    }
}

/// The local state of a [`Button`].
///
/// [`Button`]: struct.Button.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State;

impl State {
    /// Creates a new [`State`].
    ///
    /// [`State`]: struct.State.html
    pub fn new() -> State {
        State::default()
    }
}

use uikit_sys::{
    UIButton,
    IUIButton,
    UIScreen,
    IUIScreen,
    UIButtonType_UIButtonTypePlain,
    UIView_UIViewGeometry,
    ICALayer,
    IUIView,
    UIView,
    NSString,
    NSString_NSStringExtensionMethods,
    NSUTF8StringEncoding,
    IUIControl,
    id,
};
use std::ffi::CString;
use std::convert::TryInto;

impl<'a, Message> Widget<Message> for Button<'a, Message>
where
    Message: 'static + Clone,
{
    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.content.hash_layout(state);
    }
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }
    fn get_widget_type(&self) -> WidgetType {
        WidgetType::Button
    }

    fn update(&self, current_node: &mut WidgetNode, root_view: Option<UIView>) {
        match &current_node.widget_type {
            WidgetType::Button => {
            },
            other => {
                debug!("Updating from {:?}, to {:?}", other, self.get_widget_type());
                current_node.drop_from_ui();
                let new_node = self.build_uiview(root_view.is_some());
                if let Some(root_view) = root_view {
                    new_node.draw(root_view);
                }
                *current_node = new_node;

            }
        }
    }

    fn build_uiview(&self, is_root: bool) -> WidgetNode {
        let button = unsafe {
            let button = UIButton(UIButton::buttonWithType_(UIButtonType_UIButtonTypePlain));
            if is_root {
                let screen = UIScreen::mainScreen();
                let frame = screen.bounds();
                button.setFrame_(frame);
            }
            let text = String::from("THIS IS A BUTTON");
            let text = NSString(
                NSString::alloc().initWithBytes_length_encoding_(
                    CString::new(text.as_str())
                    .expect("CString::new failed")
                    .as_ptr()
                    as *mut std::ffi::c_void,
                    text.len().try_into().unwrap(),
                    NSUTF8StringEncoding,
                ),
            );
            button.setTitle_forState_(text, uikit_sys::UIControlState_UIControlStateNormal);
            let layer = button.layer();
            layer.setBorderWidth_(3.0);

            let on_change = EventHandler::new(button.0);
            button.addTarget_action_forControlEvents_(
                on_change.id,
                sel!(sendEvent),
                uikit_sys::UIControlEvents_UIControlEventTouchDown,
            );
            button
        };
        WidgetNode::new(button.0, WidgetType::Button, self.get_my_hash())
    }
    fn on_widget_event(
        &mut self,
        widget_event: WidgetEvent,
        messages: &mut Vec<Message>,
        widget_node: &WidgetNode,
    ) {
        debug!(
            "BUTTON on_widget_event for text input: widget_event.id: {:x} for widget_id: {:?}, widget_node.view_id {:?}",
            widget_event.id,
            widget_event.widget_id,
            widget_node.view_id,
        );
        if widget_event.id as id == widget_node.view_id {
            if let Some(message) = self.on_press.clone() {
                messages.push(message);
            }
        }
    }
}

impl<'a, Message> From<Button<'a, Message>> for Element<'a, Message>
where
    Message: 'static + Clone,
{
    fn from(button: Button<'a, Message>) -> Element<'a, Message> {
        Element::new(button)
    }
}
