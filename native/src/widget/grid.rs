//! Write some text for your users to read.
use crate::{
    layout::{Limits, Node},
    Element, Hasher, Layout, Length, Point, Size, Widget,
};
use std::{any::TypeId, hash::Hash, iter};

/// A container that distributes its contents in a grid.
///
/// # Example
///
/// ```
/// # use iced_native::{renderer::Null, Element, Grid as NativeGrid, Text};
/// # type Grid<'a> = NativeGrid<'a, (), Null>;
/// Grid::with_columns(2)
///     .push(Text::new("First row, first column"))
///     .push(Text::new("First row, second column"))
///     .push(Text::new("Second row, first column"))
///     .push(Text::new("Second row, second column"));
/// ```
#[allow(missing_debug_implementations)]
pub struct Grid<'a, Message, Renderer> {
    strategy: Strategy,
    elements: Vec<Element<'a, Message, Renderer>>,
}

enum Strategy {
    Columns(usize),
    ColumnWidth(u16),
}

impl<'a, Message, Renderer> Grid<'a, Message, Renderer> {
    /// Create a new empty [`Grid`].
    /// Elements will be layed out in a specific amount of columns.
    ///
    /// [`Grid`]: struct.Grid.html
    pub fn with_columns(columns: usize) -> Self {
        Self {
            strategy: Strategy::Columns(columns),
            elements: Vec::new(),
        }
    }

    /// Create a new empty [`Grid`].
    /// Columns will be generated to fill the given space.
    ///
    /// [`Grid`]: struct.Grid.html
    pub fn with_column_width(column_width: u16) -> Self {
        Self {
            strategy: Strategy::ColumnWidth(column_width),
            elements: Vec::new(),
        }
    }

    /// Adds an [`Element`] to the [`Grid`].
    ///
    /// [`Element`]: ../struct.Element.html
    /// [`Grid`]: struct.Grid.html
    pub fn push<E>(mut self, element: E) -> Self
    where
        E: Into<Element<'a, Message, Renderer>>,
    {
        self.elements.push(element.into());
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Grid<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    fn width(&self) -> Length {
        Length::Shrink
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(&self, renderer: &Renderer, limits: &Limits) -> Node {
        if self.elements.is_empty() {
            return Node::new(Size::ZERO);
        }

        match self.strategy {
            // find out how wide a column is by finding the widest cell in it
            Strategy::Columns(columns) => {
                if columns == 0 {
                    return Node::new(Size::ZERO);
                }

                let mut layouts = Vec::with_capacity(self.elements.len());
                let mut column_widths = Vec::<f32>::with_capacity(columns);

                for (column, element) in
                    (0..columns).cycle().zip(&self.elements)
                {
                    let layout = element.layout(renderer, &limits).size();
                    layouts.push(layout);

                    if let Some(column_width) = column_widths.get_mut(column) {
                        *column_width = column_width.max(layout.width);
                    } else {
                        column_widths.insert(column, layout.width);
                    }
                }

                let column_aligns = iter::once(&0.)
                    .chain(column_widths.iter())
                    .scan(0., |state, width| {
                        *state += width;
                        Some(*state)
                    });
                let grid_width = column_widths.iter().sum();

                build_grid(
                    columns,
                    column_aligns,
                    layouts.into_iter(),
                    grid_width,
                )
            }
            // find number of columns by checking how many can fit
            Strategy::ColumnWidth(column_width) => {
                let column_limits = limits.width(Length::Units(column_width));
                let column_width: f32 = column_width.into();
                let max_width = limits.max().width;
                let columns = (max_width / column_width).floor() as usize;

                let layouts = self.elements.iter().map(|element| {
                    element.layout(renderer, &column_limits).size()
                });
                let column_aligns = iter::successors(Some(0.), |width| {
                    Some(width + column_width)
                });
                let grid_width = (columns as f32) * column_width;

                build_grid(columns, column_aligns, layouts, grid_width)
            }
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        renderer.draw(defaults, layout, cursor_position, &self.elements)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        TypeId::of::<Grid<'_, (), ()>>().hash(state);

        for element in &self.elements {
            element.hash_layout(state);
        }
    }
}

fn build_grid(
    columns: usize,
    column_aligns: impl Iterator<Item = f32> + Clone,
    layouts: impl Iterator<Item = Size> + ExactSizeIterator,
    grid_width: f32,
) -> Node {
    let mut nodes = Vec::with_capacity(layouts.len());
    let mut grid_height = 0.;
    let mut row_height = 0.;

    for ((column, column_align), size) in
        (0..columns).zip(column_aligns).cycle().zip(layouts)
    {
        if column == 0 {
            grid_height += row_height;
            row_height = 0.;
        }

        let mut node = Node::new(size);
        node.move_to(Point::new(column_align, grid_height));
        nodes.push(node);
        row_height = row_height.max(size.height);
    }

    grid_height += row_height;

    Node::with_children(Size::new(grid_width, grid_height), nodes)
}

/// The renderer of a [`Grid`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use [`Grid`] in your [`UserInterface`].
///
/// [`Grid`]: struct.Grid.html
/// [renderer]: ../../renderer/index.html
/// [`UserInterface`]: ../../struct.UserInterface.html
pub trait Renderer: crate::Renderer {
    /// Draws a [`Grid`].
    ///
    /// It receives:
    /// - the list [`Element`]s
    /// - the [`Layout`] of these elements
    /// - the cursor position
    ///
    /// [`Grid`]: struct.Grid.html
    /// [`Element`]: ../struct.Element.html
    /// [`Layout`]: ../layout/struct.Layout.html
    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        elements: &[Element<'_, Message, Self>],
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<Grid<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer,
    Message: 'static,
{
    fn from(
        grid: Grid<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(grid)
    }
}
