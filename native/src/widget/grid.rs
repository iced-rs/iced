//! Write some text for your users to read.
use crate::{
    layout::{
        flex::{self, Axis},
        Limits, Node,
    },
    Align, Element, Hasher, Layout, Length, Point, Size, Widget,
};
use std::{any::TypeId, hash::Hash, iter};

/// A container that distributes its contents in a grid.
///
/// # Example
///
/// ```
/// # use iced_native::{renderer::Null, Element, Grid as NativeGrid, Text};
/// # type Grid<'a> = NativeGrid<'a, (), Null>;
/// Grid::with_children(
///     vec![
///         Element::from(Text::new("First row, first column")),
///         Element::from(Text::new("First row, second column")),
///         Element::from(Text::new("Second row, first column")),
///         Element::from(Text::new("Second row, second column")),
///     ],
/// )
/// .columns(2);
/// ```
#[allow(missing_debug_implementations)]
pub struct Grid<'a, Message, Renderer> {
    columns: Option<usize>,
    column_width: Option<u16>,
    elements: Vec<Element<'a, Message, Renderer>>,
}

impl<'a, Message, Renderer> Grid<'a, Message, Renderer> {
    /// Create a new empty [`Grid`].
    ///
    /// [`Grid`]: struct.Grid.html
    pub fn new() -> Self {
        Self::with_children(Vec::new())
    }

    /// Create a new [`Grid`] with the given [`Element`]s.
    ///
    /// [`Grid`]: struct.Grid.html
    /// [`Element`]: ../struct.Element.html
    pub fn with_children(
        elements: Vec<Element<'a, Message, Renderer>>,
    ) -> Self {
        Self {
            columns: None,
            column_width: None,
            elements,
        }
    }

    /// Sets a fixed amount of columns for the [`Grid`].
    ///
    /// [`Grid`]: struct.Grid.html
    pub fn columns(mut self, columns: usize) -> Self {
        if columns == 0 {
            self.columns = None;
        } else {
            self.columns = Some(columns);
        }

        self
    }

    /// Sets the width of columns for the [`Grid`].
    ///
    /// [`Grid`]: struct.Grid.html
    pub fn column_width(mut self, column_width: u16) -> Self {
        self.column_width = Some(column_width);
        self
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
        if self.columns.is_some() {
            Length::Shrink
        } else {
            Length::Fill
        }
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(&self, renderer: &Renderer, limits: &Limits) -> Node {
        if self.elements.is_empty() {
            Node::new(Size::ZERO)
        } else {
            let column_limits = if let Some(column_width) = self.column_width {
                limits.width(Length::Units(column_width))
            } else {
                *limits
            };

            // if we have a given number of columns, we can find out how
            // wide a column is by finding the widest cell in it
            if let Some(columns) = self.columns {
                // store calculated layout sizes
                let mut layouts = Vec::with_capacity(self.elements.len());
                // store width of each column
                let mut column_widths = Vec::<f32>::with_capacity(columns);

                for (column, element) in
                    (0..columns).cycle().zip(&self.elements)
                {
                    let layout =
                        element.layout(renderer, &column_limits).size();
                    layouts.push(layout);

                    if let Some(column_width) = column_widths.get_mut(column) {
                        *column_width = column_width.max(layout.width);
                    } else {
                        column_widths.insert(column, layout.width);
                    }
                }

                // list of x alignments for every column
                let column_aligns = iter::once(&0.)
                    .chain(column_widths.iter().take(column_widths.len() - 1))
                    .scan(0., |state, width| {
                        *state += width;
                        Some(*state)
                    });

                let mut nodes = Vec::with_capacity(self.elements.len());
                let mut grid_height = 0.;
                let mut row_height = 0.;

                for ((column, column_align), size) in
                    column_aligns.enumerate().cycle().zip(layouts)
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
                let grid_width = column_widths.into_iter().sum();

                Node::with_children(Size::new(grid_width, grid_height), nodes)
            // if we have `column_width` but no `columns`, calculate number of
            // columns by checking how many can fit
            } else if let Some(column_width) = self.column_width {
                let column_width: f32 = column_width.into();
                let max_width = limits.max().width;
                let columns = (max_width / column_width).floor() as usize;
                let mut nodes = Vec::with_capacity(self.elements.len());
                let mut grid_height = 0.;
                let mut row_height = 0.;

                for (column, element) in
                    (0..columns).cycle().zip(&self.elements)
                {
                    if column == 0 {
                        grid_height += row_height;
                        row_height = 0.;
                    }

                    let size = element.layout(renderer, &column_limits).size();
                    let mut node = Node::new(size);
                    node.move_to(Point::new(
                        column as f32 * column_width,
                        grid_height,
                    ));
                    nodes.push(node);
                    row_height = row_height.max(size.height);
                }

                grid_height += row_height;
                let grid_width = (columns as f32) * column_width;

                Node::with_children(Size::new(grid_width, grid_height), nodes)
            // if we didn't define `columns` and `column_width` just put them
            // horizontally next to each other
            } else {
                flex::resolve(
                    Axis::Horizontal,
                    renderer,
                    &limits,
                    0.,
                    0.,
                    Align::Start,
                    &self.elements,
                )
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
