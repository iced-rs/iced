use std::hash::Hash;
use crate::{
    Color,
    //    css, Bus, Color, Css,
    Element,
    Font,
    HorizontalAlignment,
    Length,
    VerticalAlignment,
    Widget,
    Hasher,
};

/// A paragraph of text.
///
/// # Example
///
/// ```
/// # use iced_web::Text;
///
/// Text::new("I <3 iced!")
///     .size(40);
/// ```
#[derive(Debug, Clone)]
pub struct Text {
    content: String,
    size: Option<u16>,
    color: Option<Color>,
    font: Font,
    width: Length,
    height: Length,
    horizontal_alignment: HorizontalAlignment,
    vertical_alignment: VerticalAlignment,
}

impl Text {
    /// Create a new fragment of [`Text`] with the given contents.
    ///
    /// [`Text`]: struct.Text.html
    pub fn new<T: Into<String>>(label: T) -> Self {
        Text {
            content: label.into(),
            size: None,
            color: None,
            font: Font::Default,
            width: Length::Shrink,
            height: Length::Shrink,
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
        }
    }

    /// Sets the size of the [`Text`].
    ///
    /// [`Text`]: struct.Text.html
    pub fn size(mut self, size: u16) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets the [`Color`] of the [`Text`].
    ///
    /// [`Text`]: struct.Text.html
    /// [`Color`]: ../../struct.Color.html
    pub fn color<C: Into<Color>>(mut self, color: C) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Sets the [`Font`] of the [`Text`].
    ///
    /// [`Text`]: struct.Text.html
    /// [`Font`]: ../../struct.Font.html
    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    /// Sets the width of the [`Text`] boundaries.
    ///
    /// [`Text`]: struct.Text.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Text`] boundaries.
    ///
    /// [`Text`]: struct.Text.html
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the [`HorizontalAlignment`] of the [`Text`].
    ///
    /// [`Text`]: struct.Text.html
    /// [`HorizontalAlignment`]: enum.HorizontalAlignment.html
    pub fn horizontal_alignment(
        mut self,
        alignment: HorizontalAlignment,
    ) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    /// Sets the [`VerticalAlignment`] of the [`Text`].
    ///
    /// [`Text`]: struct.Text.html
    /// [`VerticalAlignment`]: enum.VerticalAlignment.html
    pub fn vertical_alignment(mut self, alignment: VerticalAlignment) -> Self {
        self.vertical_alignment = alignment;
        self
    }
}
use std::convert::TryInto;
use std::ffi::CString;
use uikit_sys::{
    CGPoint, CGRect, CGSize, INSObject, IUIColor, IUILabel,
    NSString, NSString_NSStringExtensionMethods, UIColor, UILabel, UIView,
    UIView_UIViewGeometry, UIView_UIViewHierarchy,
};

impl<'a, Message> Widget<Message> for Text {
    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.content.hash(state);
        self.size.hash(state);
        self.width.hash(state);
        self.height.hash(state);
    }

    fn draw(&mut self, parent: UIView) {
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
    }
}

impl<'a, Message> From<Text> for Element<'a, Message> {
    fn from(text: Text) -> Element<'a, Message> {
        Element::new(text)
    }
}
