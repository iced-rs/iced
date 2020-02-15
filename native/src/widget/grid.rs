//! Write some text for your users to read.
use crate::{
    layout::{self, Node},
    Element, Hasher, Layout, Length, Point, Size, Widget,
};
use std::iter;

/// A container that distributes its contents in a grid.
///
/// # Example
///
/// ```
/// # use iced_native::{renderer::Null, Element, Grid as NativeGrid, Text};
/// # type Grid<'a> = NativeGrid<'a, (), Null>;
/// Grid::from_vec(
///     2,
///     vec![
///         Element::from(Text::new("First row, first column")),
///         Element::from(Text::new("First row, second column")),
///         Element::from(Text::new("Second row, first column")),
///         Element::from(Text::new("Second row, second column")),
///     ],
/// );
/// ```
#[allow(missing_debug_implementations)]
pub struct Grid<'a, Message, Renderer> {
    columns: usize,
    elements: Vec<Element<'a, Message, Renderer>>,
}

impl<'a, Message, Renderer> Grid<'a, Message, Renderer> {
    /// Create a new empty [`Grid`].
    ///
    /// [`Grid`]: struct.Grid.html
    pub fn new(columns: usize) -> Self {
        Self::from_vec(columns, Vec::new())
    }

    /// Create a new [`Grid`] with the given [`Element`]s.
    ///
    /// [`Grid`]: struct.Grid.html
    /// [`Element`]: ../struct.Element.html
    pub fn from_vec(
        columns: usize,
        elements: Vec<Element<'a, Message, Renderer>>,
    ) -> Self {
        Self { columns, elements }
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

    /// Adds [`Element`]s to the [`Grid`].
    ///
    /// [`Element`]: ../struct.Element.html
    /// [`Grid`]: struct.Grid.html
    pub fn push_vec(
        mut self,
        mut elements: Vec<Element<'a, Message, Renderer>>,
    ) -> Self {
        self.elements.append(&mut elements);
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

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        // store calculated layout sizes
        let mut layouts = Vec::with_capacity(self.elements.len());
        // store width of each column
        let mut column_widths = Vec::<f32>::with_capacity(self.columns);

        // find out how wide a column is by finding the widest cell in it
        for (column, element) in (0..self.columns).cycle().zip(&self.elements) {
            let layout = element.layout(renderer, limits).size();
            layouts.push(layout);

            if let Some(column_width) = column_widths.get_mut(column) {
                *column_width = column_width.max(layout.width);
            } else {
                column_widths.insert(column, layout.width);
            }
        }

        // list of x alignments for every column
        let column_aligns = iter::once(&0.)
            .chain(column_widths.iter().take(self.columns - 1))
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
