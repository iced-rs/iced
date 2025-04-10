//! Distribute content on a grid.
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::{Operation, Tree};
use crate::core::{
    Clipboard, Element, Event, Length, Pixels, Rectangle, Shell, Size, Vector,
    Widget,
};

/// A container that distributes its contents on a responsive grid.
#[allow(missing_debug_implementations)]
pub struct Grid<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer> {
    spacing: f32,
    columns: Constraint,
    width: Option<Pixels>,
    ratio: Option<Pixels>,
    children: Vec<Element<'a, Message, Theme, Renderer>>,
}

enum Constraint {
    MaxWidth(Pixels),
    Amount(usize),
}

impl<'a, Message, Theme, Renderer> Grid<'a, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
{
    /// Creates an empty [`Grid`].
    pub fn new() -> Self {
        Self::from_vec(Vec::new())
    }

    /// Creates a [`Grid`] with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self::from_vec(Vec::with_capacity(capacity))
    }

    /// Creates a [`Grid`] with the given elements.
    pub fn with_children(
        children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        let iterator = children.into_iter();

        Self::with_capacity(iterator.size_hint().0).extend(iterator)
    }

    /// Creates a [`Grid`] from an already allocated [`Vec`].
    pub fn from_vec(
        children: Vec<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            spacing: 0.0,
            columns: Constraint::Amount(3),
            width: None,
            ratio: None,
            children,
        }
    }

    /// Sets the spacing _between_ cells in the [`Grid`].
    pub fn spacing(mut self, amount: impl Into<Pixels>) -> Self {
        self.spacing = amount.into().0;
        self
    }

    /// Sets the width of the [`Grid`] in [`Pixels`].
    ///
    /// By default, a [`Grid`] will [`Fill`] its parent.
    ///
    /// [`Fill`]: Length::Fill
    pub fn width(mut self, width: impl Into<Pixels>) -> Self {
        self.width = Some(width.into());
        self
    }

    /// Sets the amount of columns in the [`Grid`].
    pub fn columns(mut self, column: usize) -> Self {
        self.columns = Constraint::Amount(column);
        self
    }

    /// Makes the amount of columns dynamic in the [`Grid`], never
    /// exceeding the provided `max_width`.
    pub fn fluid(mut self, max_width: impl Into<Pixels>) -> Self {
        self.columns = Constraint::MaxWidth(max_width.into());
        self
    }

    /// Sets the amount of horizontal pixels per each vertical pixel of a cell in the [`Grid`].
    pub fn ratio(mut self, ratio: impl Into<Pixels>) -> Self {
        self.ratio = Some(ratio.into());
        self
    }

    /// Adds an [`Element`] to the [`Grid`].
    pub fn push(
        mut self,
        child: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        self.children.push(child.into());
        self
    }

    /// Adds an element to the [`Grid`], if `Some`.
    pub fn push_maybe(
        self,
        child: Option<impl Into<Element<'a, Message, Theme, Renderer>>>,
    ) -> Self {
        if let Some(child) = child {
            self.push(child)
        } else {
            self
        }
    }

    /// Extends the [`Grid`] with the given children.
    pub fn extend(
        self,
        children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        children.into_iter().fold(self, Self::push)
    }
}

impl<Message, Renderer> Default for Grid<'_, Message, Renderer>
where
    Renderer: crate::core::Renderer,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message, Theme, Renderer: crate::core::Renderer>
    FromIterator<Element<'a, Message, Theme, Renderer>>
    for Grid<'a, Message, Theme, Renderer>
{
    fn from_iter<
        T: IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
    >(
        iter: T,
    ) -> Self {
        Self::with_children(iter)
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Grid<'_, Message, Theme, Renderer>
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
            width: self
                .width
                .map(|pixels| Length::Fixed(pixels.0))
                .unwrap_or(Length::Fill),
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = self.size();
        let limits = limits.width(size.width).height(size.height);
        let available = limits.max();

        let cells_per_row = match self.columns {
            // width = n * (cell + spacing) - spacing, given n > 0
            Constraint::MaxWidth(pixels) => ((available.width + self.spacing)
                / (pixels.0 + self.spacing))
                .ceil() as usize,
            Constraint::Amount(amount) => amount,
        };

        let total_rows = self.children.len() / cells_per_row;

        let cell_width = (available.width
            - self.spacing * (cells_per_row - 1) as f32)
            / cells_per_row as f32;

        let cell_height = if let Some(ratio) = self.ratio {
            cell_width / ratio.0
        } else if available.height.is_finite() {
            available.height / total_rows as f32
        } else {
            f32::INFINITY
        };

        let cell_limits = layout::Limits::new(
            Size::new(cell_width, 0.0),
            Size::new(cell_width, cell_height),
        );

        let mut nodes = Vec::new();
        let mut x = 0.0;
        let mut y = 0.0;

        for (i, (child, tree)) in
            self.children.iter().zip(&mut tree.children).enumerate()
        {
            let node = child
                .as_widget()
                .layout(tree, renderer, &cell_limits)
                .move_to((x, y));

            x += node.size().width + self.spacing;

            if (i + 1) % cells_per_row == 0 {
                y += cell_height + self.spacing;
                x = 0.0;
            }

            nodes.push(node);
        }

        if x == 0.0 {
            y -= self.spacing;
        } else {
            y += cell_height;
        }

        layout::Node::with_children(Size::new(available.width, y), nodes)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.children
                .iter()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget()
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
        for ((child, state), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child.as_widget_mut().update(
                state, event, layout, cursor, renderer, clipboard, shell,
                viewport,
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
            .map(|((child, state), layout)| {
                child.as_widget().mouse_interaction(
                    state, layout, cursor, viewport, renderer,
                )
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
        if let Some(viewport) = layout.bounds().intersection(viewport) {
            for ((child, state), layout) in self
                .children
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
                .filter(|(_, layout)| layout.bounds().intersects(&viewport))
            {
                child.as_widget().draw(
                    state, renderer, theme, style, layout, cursor, &viewport,
                );
            }
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        overlay::from_children(
            &mut self.children,
            tree,
            layout,
            renderer,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Grid<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: crate::core::Renderer + 'a,
{
    fn from(row: Grid<'a, Message, Theme, Renderer>) -> Self {
        Self::new(row)
    }
}
