//! Distribute content vertically.
use crate::core::alignment::{self, Alignment};
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::{Operation, Tree};
use crate::core::{
    Clipboard, Element, Event, Layout, Length, Padding, Pixels, Rectangle, Shell, Size, Vector,
    Widget,
};

/// A container that distributes its contents vertically.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::{button, column};
///
/// #[derive(Debug, Clone)]
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     column![
///         "I am on top!",
///         button("I am in the center!"),
///         "I am below.",
///     ].into()
/// }
/// ```
pub struct Column<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer> {
    spacing: f32,
    padding: Padding,
    width: Length,
    height: Length,
    max_width: f32,
    align: Alignment,
    clip: bool,
    children: Vec<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> Column<'a, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
{
    /// Creates an empty [`Column`].
    pub fn new() -> Self {
        Self::from_vec(Vec::new())
    }

    /// Creates a [`Column`] with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self::from_vec(Vec::with_capacity(capacity))
    }

    /// Creates a [`Column`] with the given elements.
    pub fn with_children(
        children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        let iterator = children.into_iter();

        Self::with_capacity(iterator.size_hint().0).extend(iterator)
    }

    /// Creates a [`Column`] from an already allocated [`Vec`].
    ///
    /// Keep in mind that the [`Column`] will not inspect the [`Vec`], which means
    /// it won't automatically adapt to the sizing strategy of its contents.
    ///
    /// If any of the children have a [`Length::Fill`] strategy, you will need to
    /// call [`Column::width`] or [`Column::height`] accordingly.
    pub fn from_vec(children: Vec<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            spacing: 0.0,
            padding: Padding::ZERO,
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: f32::INFINITY,
            align: Alignment::Start,
            clip: false,
            children,
        }
    }

    /// Sets the vertical spacing _between_ elements.
    ///
    /// Custom margins per element do not exist in iced. You should use this
    /// method instead! While less flexible, it helps you keep spacing between
    /// elements consistent.
    pub fn spacing(mut self, amount: impl Into<Pixels>) -> Self {
        self.spacing = amount.into().0;
        self
    }

    /// Sets the [`Padding`] of the [`Column`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the width of the [`Column`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Column`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the maximum width of the [`Column`].
    pub fn max_width(mut self, max_width: impl Into<Pixels>) -> Self {
        self.max_width = max_width.into().0;
        self
    }

    /// Sets the horizontal alignment of the contents of the [`Column`] .
    pub fn align_x(mut self, align: impl Into<alignment::Horizontal>) -> Self {
        self.align = Alignment::from(align.into());
        self
    }

    /// Sets whether the contents of the [`Column`] should be clipped on
    /// overflow.
    pub fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    /// Adds an element to the [`Column`].
    pub fn push(mut self, child: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let child = child.into();
        let child_size = child.as_widget().size_hint();

        if !child_size.is_void() {
            self.width = self.width.enclose(child_size.width);
            self.height = self.height.enclose(child_size.height);
            self.children.push(child);
        }

        self
    }

    /// Extends the [`Column`] with the given children.
    pub fn extend(
        self,
        children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        children.into_iter().fold(self, Self::push)
    }

    /// Turns the [`Column`] into a [`Wrapping`] column.
    ///
    /// The original alignment of the [`Column`] is preserved per column wrapped.
    pub fn wrap(self) -> Wrapping<'a, Message, Theme, Renderer> {
        Wrapping {
            column: self,
            horizontal_spacing: None,
            align_y: alignment::Vertical::Top,
        }
    }
}

impl<Message, Renderer> Default for Column<'_, Message, Renderer>
where
    Renderer: crate::core::Renderer,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message, Theme, Renderer: crate::core::Renderer>
    FromIterator<Element<'a, Message, Theme, Renderer>> for Column<'a, Message, Theme, Renderer>
{
    fn from_iter<T: IntoIterator<Item = Element<'a, Message, Theme, Renderer>>>(iter: T) -> Self {
        Self::with_children(iter)
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Column<'_, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.max_width(self.max_width);

        layout::flex::resolve(
            layout::flex::Axis::Vertical,
            renderer,
            &limits,
            self.width,
            self.height,
            self.padding,
            self.spacing,
            self.align,
            &mut self.children,
            &mut tree.children,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.children
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget_mut()
                        .operate(state, layout, renderer, operation);
                });
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        for ((child, tree), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child.as_widget_mut().update(
                tree, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, tree), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        if let Some(clipped_viewport) = layout.bounds().intersection(viewport) {
            let viewport = if self.clip {
                &clipped_viewport
            } else {
                viewport
            };

            for ((child, tree), layout) in self
                .children
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
                .filter(|(_, layout)| layout.bounds().intersects(viewport))
            {
                child
                    .as_widget()
                    .draw(tree, renderer, theme, style, layout, cursor, viewport);
            }
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        overlay::from_children(
            &mut self.children,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Column<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: crate::core::Renderer + 'a,
{
    fn from(column: Column<'a, Message, Theme, Renderer>) -> Self {
        Self::new(column)
    }
}

/// A [`Column`] that wraps its contents.
///
/// Create a [`Column`] first, and then call [`Column::wrap`] to
/// obtain a [`Column`] that wraps its contents.
///
/// The original alignment of the [`Column`] is preserved per column wrapped.
#[allow(missing_debug_implementations)]
pub struct Wrapping<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer> {
    column: Column<'a, Message, Theme, Renderer>,
    horizontal_spacing: Option<f32>,
    align_y: alignment::Vertical,
}

impl<Message, Theme, Renderer> Wrapping<'_, Message, Theme, Renderer> {
    /// Sets the horizontal spacing _between_ columns.
    pub fn horizontal_spacing(mut self, amount: impl Into<Pixels>) -> Self {
        self.horizontal_spacing = Some(amount.into().0);
        self
    }

    /// Sets the vertical alignment of the wrapping [`Column`].
    pub fn align_x(mut self, align_y: impl Into<alignment::Vertical>) -> Self {
        self.align_y = align_y.into();
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Wrapping<'_, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        self.column.children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.column.diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.column.size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits
            .width(self.column.width)
            .height(self.column.height)
            .shrink(self.column.padding);

        let child_limits = limits.loose();
        let spacing = self.column.spacing;
        let horizontal_spacing = self.horizontal_spacing.unwrap_or(spacing);
        let max_height = limits.max().height;

        let mut children: Vec<layout::Node> = Vec::new();
        let mut intrinsic_size = Size::ZERO;
        let mut column_start = 0;
        let mut column_width = 0.0;
        let mut x = 0.0;
        let mut y = 0.0;

        let align_factor = match self.column.align {
            Alignment::Start => 0.0,
            Alignment::Center => 2.0,
            Alignment::End => 1.0,
        };

        let align_x = |column_start: std::ops::Range<usize>,
                       column_width: f32,
                       children: &mut Vec<layout::Node>| {
            if align_factor != 0.0 {
                for node in &mut children[column_start] {
                    let width = node.size().width;

                    node.translate_mut(Vector::new((column_width - width) / align_factor, 0.0));
                }
            }
        };

        for (i, child) in self.column.children.iter_mut().enumerate() {
            let node = child
                .as_widget_mut()
                .layout(&mut tree.children[i], renderer, &child_limits);

            let child_size = node.size();

            if y != 0.0 && y + child_size.height > max_height {
                intrinsic_size.height = intrinsic_size.height.max(y - spacing);

                align_x(column_start..i, column_width, &mut children);

                x += column_width + horizontal_spacing;
                y = 0.0;
                column_start = i;
                column_width = 0.0;
            }

            column_width = column_width.max(child_size.width);

            children
                .push(node.move_to((x + self.column.padding.left, y + self.column.padding.top)));

            y += child_size.height + spacing;
        }

        if y != 0.0 {
            intrinsic_size.height = intrinsic_size.height.max(y - spacing);
        }

        intrinsic_size.width = x + column_width;
        align_x(column_start..children.len(), column_width, &mut children);

        let align_factor = match self.align_y {
            alignment::Vertical::Top => 0.0,
            alignment::Vertical::Center => 2.0,
            alignment::Vertical::Bottom => 1.0,
        };

        if align_factor != 0.0 {
            let total_height = intrinsic_size.height;

            let mut column_start = 0;

            for i in 0..children.len() {
                let bounds = children[i].bounds();
                let column_height = bounds.y + bounds.height;

                let next_y = children
                    .get(i + 1)
                    .map(|node| node.bounds().y)
                    .unwrap_or_default();

                if next_y == 0.0 {
                    let translation =
                        Vector::new(0.0, (total_height - column_height) / align_factor);

                    for node in &mut children[column_start..=i] {
                        node.translate_mut(translation);
                    }

                    column_start = i + 1;
                }
            }
        }

        let size = limits.resolve(self.column.width, self.column.height, intrinsic_size);

        layout::Node::with_children(size.expand(self.column.padding), children)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.column.operate(tree, layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.column.update(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.column
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.column
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.column
            .overlay(tree, layout, renderer, viewport, translation)
    }
}

impl<'a, Message, Theme, Renderer> From<Wrapping<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: crate::core::Renderer + 'a,
{
    fn from(column: Wrapping<'a, Message, Theme, Renderer>) -> Self {
        Self::new(column)
    }
}
