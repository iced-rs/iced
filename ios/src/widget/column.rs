use crate::{Align, event::WidgetEvent, Element, Length, Widget, Hasher, layout, WidgetPointers};
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
    UIView,
    id,
    UIStackView,
    IUIStackView,
    UIView_UIViewRendering,
    CGRect, CGPoint, CGSize,
    INSObject,
    UIColor, IUIColor,
    UIView_UIViewHierarchy,
    UIView_UIViewGeometry,

    NSLayoutDimension,
    INSLayoutDimension,
    NSLayoutConstraint,
    INSLayoutConstraint,
    UIView_UIViewLayoutConstraintCreation,
    UIStackViewDistribution_UIStackViewDistributionFill,
    UITextView,
    IUITextView,


    UILayoutConstraintAxis_UILayoutConstraintAxisVertical,
    UIStackViewAlignment_UIStackViewAlignmentCenter,
};

impl<'a, Message> Widget<Message> for Column<'a, Message>
where
    Message: 'static,
{
    fn on_widget_event(
        &mut self,
        event: WidgetEvent,
        //_layout: Layout<'_>,
        //_cursor_position: Point,
        messages: &mut Vec<Message>,
        widget_pointers: &WidgetPointers,
        //_renderer: &Renderer,
        //_clipboard: Option<&dyn Clipboard>,
    ) {
        debug!("on_widget_event for column");
        for i in &mut self.children {
            debug!("on_widget_event for child!");
            i.on_widget_event(event.clone(), messages, widget_pointers);
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
    fn layout(
        &self,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits
            .max_width(self.max_width)
            .max_height(self.max_height)
            .width(self.width)
            .height(self.height);

        layout::flex::resolve(
            layout::flex::Axis::Vertical,
            &limits,
            self.padding as f32,
            self.spacing as f32,
            self.align_items,
            &self.children,
        )
    }
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn draw(&mut self, parent: UIView) -> WidgetPointers {
        let stack_view = unsafe {
            let rect = CGRect {
                origin: CGPoint { x: 0.0, y: 0.0 },
                size: CGSize {
                    height: 400.0,
                    width: 400.0,
                },
            };
            let stack_view = UIStackView(UIStackView::alloc().initWithFrame_(rect));
            //let stack_view = UIStackView(UIStackView::alloc().init());
            //stack_view.setFrame_(rect);
            stack_view.setAxis_(UILayoutConstraintAxis_UILayoutConstraintAxisVertical);
            stack_view.setAlignment_(UIStackViewAlignment_UIStackViewAlignmentCenter);
            stack_view.setDistribution_(UIStackViewDistribution_UIStackViewDistributionFill);
            stack_view.setSpacing_(10.0);
            for i in &mut self.children {
                let subview = UIView(i.draw(UIView(0 as id)).root);
                debug!("SUBVIEW VALUE: {:?}, PARENT: {:?}", subview.0, parent.0);
                let layout = NSLayoutDimension(subview.heightAnchor());
                NSLayoutConstraint(layout.constraintEqualToConstant_(100.0)).setActive_(true);
                let layout = NSLayoutDimension(subview.widthAnchor());
                NSLayoutConstraint(layout.constraintEqualToConstant_(100.0)).setActive_(true);
                stack_view.addArrangedSubview_(subview.0);
            }

            /*
            let view3 = UITextView(UITextView::alloc().init());
            let layout = NSLayoutDimension(view3.heightAnchor());
            NSLayoutConstraint(layout.constraintEqualToConstant_(100.0)).setActive_(true);
            let layout = NSLayoutDimension(view3.widthAnchor());
            NSLayoutConstraint(layout.constraintEqualToConstant_(120.0)).setActive_(true);
            stack_view.addArrangedSubview_(view3.0);

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
            */

            parent.addSubview_(stack_view.0);

            //let background = UIColor(UIColor::greenColor());
            //let background = UIColor(UIColor::blackColor());
            //stack_view.setBackgroundColor_(background.0);
            stack_view
        };
        WidgetPointers {
            root: stack_view.0,
            others: Vec::new(),
            hash: 0,
            //children: None
        }
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
