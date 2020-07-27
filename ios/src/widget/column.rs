use crate::{
    event::WidgetEvent,
    widget::{RenderAction, WidgetNode, WidgetType},
    Align, Element, Hasher, Length, Widget,
};
use std::cell::RefCell;
use std::hash::Hash;
use std::rc::Rc;

use std::u32;
use uikit_sys::{
    id, CGPoint, CGRect, CGSize, INSLayoutConstraint, INSLayoutDimension,
    INSObject, IUIColor, IUIStackView, IUITextView, NSLayoutConstraint,
    NSLayoutDimension, UIColor,
    UILayoutConstraintAxis_UILayoutConstraintAxisVertical, UIStackView,
    UIStackViewAlignment_UIStackViewAlignmentCenter,
    UIStackViewDistribution_UIStackViewDistributionFill, UITextView, UIView,
    UIView_UIViewGeometry, UIView_UIViewHierarchy,
    UIView_UIViewLayoutConstraintCreation, UIView_UIViewRendering,
    UIScreen,
    IUIScreen,
};

/// A container that distributes its contents vertically.
///
/// A [`Column`] will try to fill the horizontal space of its container.
///
/// [`Column`]: struct.Column.html
#[allow(missing_debug_implementations)]
pub struct Column<'a, Message> {
    spacing: u16,
    padding: u16,
    width: Length,
    height: Length,
    max_width: u32,
    max_height: u32,
    align_items: Align,
    children: Vec<Element<'a, Message>>,
}

impl<'a, Message> Column<'a, Message> {
    /// Creates an empty [`Column`].
    ///
    /// [`Column`]: struct.Column.html
    pub fn new() -> Self {
        Self::with_children(Vec::new())
    }

    /// Creates a [`Column`] with the given elements.
    ///
    /// [`Column`]: struct.Column.html
    pub fn with_children(children: Vec<Element<'a, Message>>) -> Self {
        Column {
            spacing: 0,
            padding: 0,
            width: Length::Fill,
            height: Length::Shrink,
            max_width: u32::MAX,
            max_height: u32::MAX,
            align_items: Align::Start,
            children,
        }
    }

    /// Sets the vertical spacing _between_ elements.
    ///
    /// Custom margins per element do not exist in Iced. You should use this
    /// method instead! While less flexible, it helps you keep spacing between
    /// elements consistent.
    pub fn spacing(mut self, units: u16) -> Self {
        self.spacing = units;
        self
    }

    /// Sets the padding of the [`Column`].
    ///
    /// [`Column`]: struct.Column.html
    pub fn padding(mut self, units: u16) -> Self {
        self.padding = units;
        self
    }

    /// Sets the width of the [`Column`].
    ///
    /// [`Column`]: struct.Column.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Column`].
    ///
    /// [`Column`]: struct.Column.html
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the maximum width of the [`Column`].
    ///
    /// [`Column`]: struct.Column.html
    pub fn max_width(mut self, max_width: u32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Sets the maximum height of the [`Column`] in pixels.
    ///
    /// [`Column`]: struct.Column.html
    pub fn max_height(mut self, max_height: u32) -> Self {
        self.max_height = max_height;
        self
    }

    /// Sets the horizontal alignment of the contents of the [`Column`] .
    ///
    /// [`Column`]: struct.Column.html
    pub fn align_items(mut self, align: Align) -> Self {
        self.align_items = align;
        self
    }

    /// Adds an element to the [`Column`].
    ///
    /// [`Column`]: struct.Column.html
    pub fn push<E>(mut self, child: E) -> Self
    where
        E: Into<Element<'a, Message>>,
    {
        self.children.push(child.into());
        self
    }
}

impl<'a, Message> Widget<Message> for Column<'a, Message>
where
    Message: 'static,
{
    fn on_widget_event(
        &mut self,
        event: WidgetEvent,
        messages: &mut Vec<Message>,
        widget_node: &WidgetNode,
    ) {
        debug!(
            "on_widget_event for column for {:?} children",
            self.children.len()
        );
        match &widget_node.widget_type {
            WidgetType::Column(node_children) => {
                for (i, node) in
                    &mut self.children.iter_mut().zip(node_children)
                    {
                        i.on_widget_event(event.clone(), messages, &node.borrow());
                    }
            }
            e => {
                error!("Widget tree traversal out of sync. {:?} should be a Column!", e);
            }
        }
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.height.hash(state);
        self.max_width.hash(state);
        self.max_height.hash(state);
        self.align_items.hash(state);
        self.spacing.hash(state);

        for child in &self.children {
            child.widget.hash_layout(state);
        }
    }
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn get_widget_type(&self) -> WidgetType {
        WidgetType::Column(Vec::new())
    }
    fn get_widget_node(&self) -> WidgetNode {
        /*
        let children = self
            .children
            .into_iter()
            .map(|i| Rc::new(RefCell::new(i.get_widget_node())))
            .collect::<Vec<Rc<RefCell<WidgetNode>>>>();
        */

        let mut children = Vec::new();
        for i in &self.children {
            children.push(Rc::new(RefCell::new(i.get_widget_node())));
        }

        let mut node = WidgetNode::new(
            0 as id,
            WidgetType::Column(children),
            self.get_my_hash(),
        );
        /*
        for i in &self.children {
            node.add_child(i.get_widget_node());
        }
        */
        node
    }
    fn build_uiview(&self, is_root: bool) -> WidgetNode {
        let stack_view = unsafe {

            /*
            let rect = CGRect {
                origin: CGPoint { x: 0.0, y: 0.0 },
                size: CGSize {
                    height: 400.0,
                    width: 500.0,
                },
            };
            */
            //let stack_view =
            //    UIStackView(UIStackView::alloc().initWithFrame_(rect));
            let stack_view = UIStackView(UIStackView::alloc().init());
            if is_root {
                let screen = UIScreen(UIScreen::mainScreen());
                let frame = screen.bounds();
                stack_view.setFrame_(frame);
            }
            /*
             *
             * This is for a Row widget.
            stack_view.setAxis_(
                uikit_sys::UILayoutConstraintAxis_UILayoutConstraintAxisHorizontal,
            );
            */
            stack_view.setAxis_(
                UILayoutConstraintAxis_UILayoutConstraintAxisVertical,
            );
            stack_view.setAlignment_(UIStackViewAlignment_UIStackViewAlignmentCenter);
            stack_view.setDistribution_(
                UIStackViewDistribution_UIStackViewDistributionFill,
            );
            stack_view
        };
        let mut stackview_node = WidgetNode::new(
            stack_view.0,
            self.get_widget_type(),
            self.get_my_hash(),
        );
        for (i, val) in self.children.iter().enumerate() {
            let node = val.build_uiview(false);
            let subview = UIView(node.view_id);
            stackview_node.add_child(node);
            unsafe {
                let layout = NSLayoutDimension(subview.heightAnchor());
                NSLayoutConstraint(layout.constraintEqualToConstant_(10.0))
                    .setActive_(true);
                let layout = NSLayoutDimension(subview.widthAnchor());
                NSLayoutConstraint(layout.constraintEqualToConstant_(10.0))
                    .setActive_(true);
                stack_view.addArrangedSubview_(subview.0);
            }
        }
        stackview_node
    }
}

impl<'a, Message> From<Column<'a, Message>> for Element<'a, Message>
where
    Message: 'static,
{
    fn from(column: Column<'a, Message>) -> Element<'a, Message> {
        Element::new(column)
    }
}
