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
    widget::{WidgetNode, WidgetType, RenderAction},
    Hasher, Size,
};
use std::convert::TryInto;
use std::ffi::CString;
use uikit_sys::{
    CGPoint, CGRect, CGSize, INSObject, IUIColor, IUILabel,
    NSString, NSString_NSStringExtensionMethods, UIColor, UILabel, UIView,
    UIView_UIViewGeometry, UIView_UIViewHierarchy,
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
        WidgetType::Text
    }

    fn update_or_add(&mut self, parent: Option<UIView>, old_node: Option<WidgetNode>,) -> WidgetNode {
        /*
        match element
            .get_render_action(widget_tree.take().as_ref())
            {
                RenderAction::Add | RenderAction::Update => {
                    debug!("Adding or updating root widget {:?} with {:?}", widget_tree.as_ref(), element.get_widget_type());
                    widget_tree = Some(element.update_or_add(
                            Some(root_view),
                            widget_tree.take(),
                    ));
                }
                RenderAction::Remove => {
                    if let Some(node) = &widget_tree {
                        debug!("Removing root widget {:?} with {:?}", node, element.get_widget_type());
                        node.drop_from_ui();
                    }
                    widget_tree = Some(element.update_or_add(
                            Some(root_view),
                            None,
                    ));
                },
            }
        */
        match self.get_render_action(old_node.as_ref()) {

            RenderAction::Add => {

                let label = unsafe {
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
                    /*
                       let rect = CGRect {
                       origin: CGPoint { x: 0.0, y: 0.0 },
                       size: CGSize {
                       height: 0.0,
                       width: 0.0,
                       },
                       };
                       label.setFrame_(rect);
                       */
                    label.setAdjustsFontSizeToFitWidth_(true);
                    label.setMinimumScaleFactor_(10.0);
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
                    if let Some(parent) = parent {
                        parent.addSubview_(label.0);
                    }
                    label
                };
                WidgetNode::new(Some(label.0), self.get_widget_type())
            },
            RenderAction::Update => {
                if let Some(node) = old_node {
                    let label = unsafe {
                        let label = UILabel(node.view_id.unwrap());
                        let text = NSString(
                            NSString::alloc().initWithBytes_length_encoding_(
                                CString::new(self.content.as_str())
                                .expect("CString::new failed")
                                .as_ptr() as *mut std::ffi::c_void,
                                self.content.len().try_into().unwrap(),
                                uikit_sys::NSUTF8StringEncoding,
                            ),
                        );
                        label.setText_(text.0);
                        label
                    };
                    WidgetNode::new(Some(label.0), self.get_widget_type())
                } else {
                    WidgetNode::new(None, self.get_widget_type())
                }
            }
            RenderAction::Remove => {
                if let Some(node) = old_node {
                    node.drop_from_ui();
                    /*
                    let view = UIView(node.view_id);
                    unsafe {
                        view.removeFromSuperview();
                    }*/
                }
                WidgetNode::new(None, self.get_widget_type())
            },
        }

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

impl<Message> From<Text<Message>> for WidgetNode
{
    fn from(_text: Text<Message>) -> WidgetNode {
        WidgetNode::new(None, WidgetType::Text)
    }
}
