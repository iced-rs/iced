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
    widget::{WidgetNode, WidgetType},
    Hasher, Size,
};
use std::convert::TryInto;
use std::ffi::CString;
use uikit_sys::{
    id,
    CGPoint, CGRect, CGSize, INSObject, IUIColor, IUILabel,
    NSString, NSString_NSStringExtensionMethods,
    UIColor, UILabel, UIView, IUIView,
    UIView_UIViewGeometry, UIView_UIViewHierarchy,
    UIView_UIViewRendering,
    ICALayer,
    UIScreen, IUIScreen,
};
use std::{
    cell::RefCell,
    rc::Rc,
};
use std::marker::PhantomData;

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
pub struct Text<Message> {
    content: String,
    size: Option<u16>,
    color: Option<Color>,
    font: Font,
    width: Length,
    height: Length,
    horizontal_alignment: HorizontalAlignment,
    vertical_alignment: VerticalAlignment,
    phantom: PhantomData<Message>,
}

impl<Message> Text<Message> {
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
            phantom: PhantomData,
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

impl<Message> Widget<Message> for Text<Message> {

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.content.hash(state);
        self.size.hash(state);
        self.width.hash(state);
        self.height.hash(state);
    }

    fn get_widget_type(&self) -> WidgetType {
        WidgetType::Text(self.content.clone())
    }
    fn build_uiview(&self, is_root: bool) -> WidgetNode {
        let content = self.content.clone();
        let color = self.color.clone();
        let label = unsafe {
            let label = UILabel::alloc();
            label.init();

            let text = NSString(
                NSString::alloc().initWithBytes_length_encoding_(
                    CString::new(content.as_str())
                    .expect("CString::new failed")
                    .as_ptr() as *mut std::ffi::c_void,
                    content.len().try_into().unwrap(),
                    uikit_sys::NSUTF8StringEncoding,
                ),
            );
            label.setText_(text);
            if is_root {
                let screen = UIScreen::mainScreen();
                let frame = screen.bounds();
                label.setFrame_(frame);
            }

            let layer = label.layer();
            layer.setBorderWidth_(3.0);

            label.setAdjustsFontSizeToFitWidth_(true);
            label.setMinimumScaleFactor_(10.0);
            label.setClipsToBounds_(true);
            if let Some(color) = color {
                let background =
                    UIColor::alloc().initWithRed_green_blue_alpha_(
                        color.r.into(),
                        color.g.into(),
                        color.b.into(),
                        color.a.into(),
                    );
                label.setTextColor_(background)
            }
            label
        };
        WidgetNode::new(
            label.0,
            self.get_widget_type(),
            self.get_my_hash()
            )
    }
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

}

impl<'a, Message> From<Text<Message>> for Element<'a, Message>
where Message: 'a
{
    fn from(text: Text<Message>) -> Element<'a, Message> {
        Element::new(text)
    }
}
