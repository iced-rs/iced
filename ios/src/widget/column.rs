use crate::{
    event::WidgetEvent,
    widget::{WidgetNode, WidgetType, RenderAction,},
    Align, Element, Hasher, Length, Widget,
};
use std::hash::Hash;

use std::u32;

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
use uikit_sys::{
    id, CGPoint, CGRect, CGSize, INSLayoutConstraint, INSLayoutDimension,
    INSObject, IUIColor, IUIStackView, IUITextView, NSLayoutConstraint,
    NSLayoutDimension, UIColor,
    UILayoutConstraintAxis_UILayoutConstraintAxisVertical, UIStackView,
    UIStackViewAlignment_UIStackViewAlignmentCenter,
    UIStackViewDistribution_UIStackViewDistributionFill, UITextView, UIView,
    UIView_UIViewGeometry, UIView_UIViewHierarchy,
    UIView_UIViewLayoutConstraintCreation, UIView_UIViewRendering,
};

impl<'a, Message> Widget<Message> for Column<'a, Message>
where
    Message: 'static,
{
    fn on_widget_event(
        &mut self,
        event: WidgetEvent,
        //_layout: Layout<'_>,
        messages: &mut Vec<Message>,
        widget_node: &WidgetNode,
    ) {
        debug!("on_widget_event for column for {:?} children", self.children.len());
        for (i, node) in
            &mut self.children.iter_mut().zip(widget_node.children.iter())
        {
            debug!("on_widget_event for {:?} child", i.get_widget_type());
            i.on_widget_event(event.clone(), messages, &node.borrow());
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
        WidgetType::Column
    }

    fn update_or_add(&mut self, parent: Option<UIView>, old_node: Option<WidgetNode>,) -> WidgetNode {
        match self.get_render_action(old_node.as_ref()) {
            RenderAction::Add => {

                let stack_view = unsafe {
                    let rect = CGRect {
                        origin: CGPoint { x: 0.0, y: 0.0 },
                        size: CGSize {
                            height: 400.0,
                            width: 300.0,
                        },
                    };
                    let stack_view =
                        UIStackView(UIStackView::alloc().initWithFrame_(rect));
                    //let stack_view = UIStackView(UIStackView::alloc().init());
                    //stack_view.setFrame_(rect);
                    stack_view.setAxis_(
                        UILayoutConstraintAxis_UILayoutConstraintAxisVertical,
                    );
                    //stack_view .setAlignment_(UIStackViewAlignment_UIStackViewAlignmentCenter);
                    stack_view.setDistribution_(
                        UIStackViewDistribution_UIStackViewDistributionFill,
                    );
                    if let Some(parent) = parent {
                        parent.addSubview_(stack_view.0);
                    }

                    /*
                       let view3 = UITextView(UITextView::alloc().init());
                       let layout = NSLayoutDimension(view3.heightAnchor());
                       NSLayoutConstraint(layout.constraintEqualToConstant_(100.0)).setActive_(true);
                       let layout = NSLayoutDimension(view3.widthAnchor());
                       NSLayoutConstraint(layout.constraintEqualToConstant_(120.0)).setActive_(true);
                       stack_view.addArrangedSubview_(view3.0);
                       */

                    stack_view
                };
                let mut stackview_node =
                    WidgetNode::new(Some(stack_view.0), self.get_widget_type());
                for (i, val) in self.children.iter_mut().enumerate() {
                    let node = val.update_or_add(None, None);
                    let subview = UIView(node.view_id.unwrap());
                    stackview_node.add_child(node);
                    unsafe {
                        let layout = NSLayoutDimension(subview.heightAnchor());
                        NSLayoutConstraint(layout.constraintEqualToConstant_(100.0))
                            .setActive_(true);
                        let layout = NSLayoutDimension(subview.widthAnchor());
                        NSLayoutConstraint(layout.constraintEqualToConstant_(100.0))
                            .setActive_(true);
                        stack_view.addArrangedSubview_(subview.0);
                    }
                }
                stackview_node
            },
            RenderAction::Update => {
                /*
                if let Some(node) = old_node {
                    let stack_view = UIStackView(node.view_id);
                    let mut stackview_node =
                        WidgetNode::new(stack_view.0, WidgetType::Column);
                    for (i, val) in self.children.iter_mut().enumerate() {
                        if let Some(child) = node.children.get(i).as_deref() {
                            let node = val.update_or_add(None, Some((*child.borrow()).clone()));
                        } else {
                            let node = val.update_or_add(None, None);
                            let subview = UIView(node.view_id);
                            stackview_node.add_child(node);
                            unsafe {
                                let layout = NSLayoutDimension(subview.heightAnchor());
                                NSLayoutConstraint(layout.constraintEqualToConstant_(100.0))
                                    .setActive_(true);
                                let layout = NSLayoutDimension(subview.widthAnchor());
                                NSLayoutConstraint(layout.constraintEqualToConstant_(100.0))
                                    .setActive_(true);
                                stack_view.addArrangedSubview_(subview.0);
                            }
                        }
                    }
                    stackview_node
                } else {
                    WidgetNode::default()
                }
                */
                WidgetNode::new(None, self.get_widget_type())
            },
            RenderAction::Remove => {
                if let Some(node) = old_node {
                    node.drop_from_ui();
                }
                WidgetNode::new(None, self.get_widget_type())
            }
        }
        /*
        if let Some(old_node) = old_node {
            if self.children.len() != old_node.children.len() {
                for i in &mut self.children {
                    let node = i.update_or_add(None, None);
                    let subview = UIView(node.view_id);
                    stackview_node.add_child(node);
                    unsafe {
                        let layout = NSLayoutDimension(subview.heightAnchor());
                        NSLayoutConstraint(layout.constraintEqualToConstant_(100.0))
                            .setActive_(true);
                        let layout = NSLayoutDimension(subview.widthAnchor());
                        NSLayoutConstraint(layout.constraintEqualToConstant_(100.0))
                            .setActive_(true);
                        stack_view.addArrangedSubview_(subview.0);
                    }
                }
            } else {
                for (i, node) in
                    self.children.iter_mut().zip(old_node.children.iter())
                    {
                        if i.get_widget_type() == node.borrow().widget_type {
                        } else {
                            unsafe {
                                let view = UIView(node.borrow().view_id);
                                //view.removeFromSuperview();
                                stack_view.removeArrangedSubview_(node.borrow().view_id);
                            }
                            let node = i.update_or_add(None, Some(node.borrow()));
                            let subview = UIView(node.view_id);
                            stackview_node.add_child(node);
                            unsafe {
                                let layout = NSLayoutDimension(subview.heightAnchor());
                                NSLayoutConstraint(layout.constraintEqualToConstant_(100.0))
                                    .setActive_(true);
                                let layout = NSLayoutDimension(subview.widthAnchor());
                                NSLayoutConstraint(layout.constraintEqualToConstant_(100.0))
                                    .setActive_(true);
                                stack_view.addArrangedSubview_(subview.0);
                            }
                        }
                    }
            }
        } else {
        */
        //}
        /*
        unsafe {

            stack_view.setSpacing_(10.0);
            let view1 = UIView(UIView::alloc().init());
            view1.setBackgroundColor_(UIColor::redColor());
            let layout = NSLayoutDimension(view1.heightAnchor());
            NSLayoutConstraint(layout.constraintEqualToConstant_(100.0)).setActive_(true);

            let layout = NSLayoutDimension(view1.widthAnchor());
            NSLayoutConstraint(layout.constraintEqualToConstant_(100.0)).setActive_(true);
            stack_view.addArrangedSubview_(view1.0);

            let view2 = UIView(UIView::alloc().init());
            view2.setBackgroundColor_(UIColor::blueColor());
            let layout = NSLayoutDimension(view2.heightAnchor());
            NSLayoutConstraint(layout.constraintEqualToConstant_(100.0)).setActive_(true);
            let layout = NSLayoutDimension(view2.widthAnchor());
            NSLayoutConstraint(layout.constraintEqualToConstant_(100.0)).setActive_(true);
            stack_view.addArrangedSubview_(view2.0);
        }
        */
        //stackview_node
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
/*
impl<'a, Message> From<Column<'a, Message>> for WidgetNode {
    fn from(column: Column<'a, Message>) -> WidgetNode {
        let mut widget = WidgetNode::new(None, WidgetType::Column);
        for i in &column.children {
            widget.add_child(WidgetNode::from(i));
        }
        widget
    }
}
*/
#[test]
fn test_foo() {
    assert!(true);
}
