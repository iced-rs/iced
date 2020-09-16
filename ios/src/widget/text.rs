use crate::{
    widget::{WidgetNode, WidgetType},
    Color, Element, Font, Hasher, HorizontalAlignment, Length, Size,
    VerticalAlignment, Widget,
};
use std::convert::TryInto;
use std::ffi::CString;
use std::hash::Hash;
use std::marker::PhantomData;
use uikit_sys::{
    id, ICALayer, INSObject, IUIColor, IUILabel, IUIScreen, IUITextView,
    IUIView, NSString, NSString_NSStringExtensionMethods, NSUTF8StringEncoding,
    UIColor, UILabel, UIScreen, UITextView, UIView, UIView_UIViewGeometry,
    UIView_UIViewRendering,
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
    fn update(&self, current_node: &mut WidgetNode, root_view: Option<UIView>) {
        let mut ids_to_drop: Vec<id> = Vec::new();
        let new_text = &self.content;
        let cstr = CString::new(new_text.as_str())
            .expect("CString::new failed");
        let cstr = cstr.as_ptr() as *mut std::ffi::c_void;
            //.as_ptr()
            //as *mut std::ffi::c_void;
        if let WidgetType::Text(ref mut old_text) = &mut current_node.widget_type {

            // TODO: check/update the styles of the text
            if old_text != new_text {
                let label = UITextView(current_node.view_id);
                unsafe {
                    let text = NSString(
                        NSString::alloc().initWithBytes_length_encoding_(
                            cstr,
                            new_text.len().try_into().unwrap(),
                            NSUTF8StringEncoding,
                        ),
                    );
                    label.setText_(text.clone());
                    ids_to_drop.push(text.0);
                }
            }
            *old_text = new_text.clone();
        } else {
            current_node.drop_from_ui();
            trace!("{:?} is not a text widget! Dropping!", current_node.widget_type);
            *current_node = self.build_uiview(root_view.is_some());
        }
        for i in &ids_to_drop {
            current_node.add_related_id(*i);
        }
    }
    fn build_uiview(&self, is_root: bool) -> WidgetNode {
        let content = self.content.clone();
        let color = self.color;
        let mut ids_to_drop: Vec<id> = Vec::new();
        let cstr = CString::new(content.as_str())
            .expect("CString::new failed");
        let cstr = cstr.as_ptr() as *mut std::ffi::c_void;

        let text = unsafe {
            NSString(NSString::alloc().initWithBytes_length_encoding_(
                    cstr,
                    content.len().try_into().unwrap(),
                    NSUTF8StringEncoding,
            ))
        };
        ids_to_drop.push(text.0);
        let label = unsafe {
            let label = UILabel::alloc();
            label.init();

            label.setText_(text);
            if is_root {
                let screen = UIScreen::mainScreen();
                let frame = screen.bounds();
                label.setFrame_(frame);
            }

            // TODO: Make this a debug feature
            let layer = label.layer();
            layer.setBorderWidth_(3.0);

            label.setAdjustsFontSizeToFitWidth_(true);
            label.setMinimumScaleFactor_(10.0);
            label.setClipsToBounds_(true);

            label
        };
        if let Some(color) = color {
            let background = unsafe {UIColor::alloc()
                .initWithRed_green_blue_alpha_(
                    color.r.into(),
                    color.g.into(),
                    color.b.into(),
                    color.a.into(),
                )
            };
            ids_to_drop.push(background.0);
            unsafe {
                label.setTextColor_(background)
            }
        }

        let mut node = WidgetNode::new(
            label.0,
            self.get_widget_type(),
            self.get_my_hash(),
        );
        for i in &ids_to_drop {
            node.add_related_id(*i);
        }
        node
    }
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }
}

impl<'a, Message> From<Text<Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(text: Text<Message>) -> Element<'a, Message> {
        Element::new(text)
    }
}
