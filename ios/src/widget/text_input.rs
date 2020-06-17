
//! Display fields that can be filled with text.
//!
//! A [`TextInput`] has some local [`State`].
//!
//! [`TextInput`]: struct.TextInput.html
//! [`State`]: struct.State.html
use crate::{Element, Length, Widget};

pub use iced_style::text_input::{Style, StyleSheet};

use std::{rc::Rc, u32};

/// A field that can be filled with text.
///
/// # Example
/// ```
/// # use iced_web::{text_input, TextInput};
/// #
/// enum Message {
///     TextInputChanged(String),
/// }
///
/// let mut state = text_input::State::new();
/// let value = "Some text";
///
/// let input = TextInput::new(
///     &mut state,
///     "This is the placeholder...",
///     value,
///     Message::TextInputChanged,
/// );
/// ```
#[allow(missing_debug_implementations)]
pub struct TextInput<'a, Message> {
    _state: &'a mut State,
    placeholder: String,
    value: String,
    is_secure: bool,
    width: Length,
    max_width: u32,
    padding: u16,
    size: Option<u16>,
    on_change: Rc<Box<dyn Fn(String) -> Message>>,
    on_submit: Option<Message>,
    style_sheet: Box<dyn StyleSheet>,
}

impl<'a, Message> TextInput<'a, Message> {
    /// Creates a new [`TextInput`].
    ///
    /// It expects:
    /// - some [`State`]
    /// - a placeholder
    /// - the current value
    /// - a function that produces a message when the [`TextInput`] changes
    ///
    /// [`TextInput`]: struct.TextInput.html
    /// [`State`]: struct.State.html
    pub fn new<F>(
        state: &'a mut State,
        placeholder: &str,
        value: &str,
        on_change: F,
    ) -> Self
    where
        F: 'static + Fn(String) -> Message,
    {
        Self {
            _state: state,
            placeholder: String::from(placeholder),
            value: String::from(value),
            is_secure: false,
            width: Length::Fill,
            max_width: u32::MAX,
            padding: 0,
            size: None,
            on_change: Rc::new(Box::new(on_change)),
            on_submit: None,
            style_sheet: Default::default(),
        }
    }

    /// Converts the [`TextInput`] into a secure password input.
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn password(mut self) -> Self {
        self.is_secure = true;
        self
    }

    /// Sets the width of the [`TextInput`].
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the maximum width of the [`TextInput`].
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn max_width(mut self, max_width: u32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Sets the padding of the [`TextInput`].
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn padding(mut self, units: u16) -> Self {
        self.padding = units;
        self
    }

    /// Sets the text size of the [`TextInput`].
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn size(mut self, size: u16) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets the message that should be produced when the [`TextInput`] is
    /// focused and the enter key is pressed.
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn on_submit(mut self, message: Message) -> Self {
        self.on_submit = Some(message);
        self
    }

    /// Sets the style of the [`TextInput`].
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn style(mut self, style: impl Into<Box<dyn StyleSheet>>) -> Self {
        self.style_sheet = style.into();
        self
    }
}

use std::convert::TryInto;
use std::ffi::CString;
use uikit_sys::{
    id, CGPoint, CGRect, CGSize, INSObject, IUIColor, IUILabel,
    NSString, NSString_NSStringExtensionMethods, UIColor, UILabel, UIView,
    UIView_UIViewGeometry, UIView_UIViewHierarchy,
};

impl<'a, Message> Widget<Message> for TextInput<'a, Message>
where
    Message: 'static + Clone,
{
    fn draw(&self, parent: UIView) {
        use uikit_sys::{
            UITextView,
            IUITextView,
        };
        let input_rect = CGRect {
            origin: CGPoint {
                x: 10.0,
                y: 50.0
            },
            size: CGSize {
                width: 200.0,
                height: 200.0,
            }
        };
        unsafe {
            let input = {
                let foo = UITextView(
                    UITextView::alloc().initWithFrame_textContainer_(
                        input_rect,
                        0 as id,
                    ));
                foo
            };
            parent.addSubview_(input.0);
        }
        /*
        unsafe {
            let label = UILabel::alloc();
            let text = NSString(
                NSString::alloc().initWithBytes_length_encoding_(
                    CString::new(self.content.as_str())
                        .expect("CString::new failed")
                        .as_ptr() as *mut std::ffi::c_void,
                    self.content.len().try_into().unwrap(),
                    uikit_sys::NSUTF8StringEncoding,
                ),
            );
            label.init();
            label.setText_(text.0);
            let rect = CGRect {
                origin: CGPoint { x: 0.0, y: 0.0 },
                size: CGSize {
                    height: 20.0,
                    width: 500.0,
                },
            };
            label.setAdjustsFontSizeToFitWidth_(true);
            label.setMinimumScaleFactor_(100.0);
            label.setFrame_(rect);
            if let Some(color) = self.color {
                let background =
                    UIColor(UIColor::alloc().initWithRed_green_blue_alpha_(
                        color.r.into(),
                        color.g.into(),
                        color.b.into(),
                        color.a.into(),
                    ));
                label.setTextColor_(background.0)
            }
            label.setFrame_(rect);
            parent.addSubview_(label.0);
        };
        */
    }
}

impl<'a, Message> From<TextInput<'a, Message>> for Element<'a, Message>
where
    Message: 'static + Clone,
{
    fn from(text_input: TextInput<'a, Message>) -> Element<'a, Message> {
        Element::new(text_input)
    }
}

/// The state of a [`TextInput`].
///
/// [`TextInput`]: struct.TextInput.html
#[derive(Debug, Clone, Copy, Default)]
pub struct State;

impl State {
    /// Creates a new [`State`], representing an unfocused [`TextInput`].
    ///
    /// [`State`]: struct.State.html
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`State`], representing a focused [`TextInput`].
    ///
    /// [`State`]: struct.State.html
    pub fn focused() -> Self {
        // TODO
        Self::default()
    }
}
