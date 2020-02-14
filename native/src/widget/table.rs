//! Write some text for your users to read.
use crate::{
    layout::{self, Node},
    Element, Hasher, Layout, Length, Point, Size, Widget,
};
use std::iter;

/// A table.
///
/// # Example
///
/// ```
/// # use iced_native::{Table, Text};
/// #
/// Table::from_vec(vec![
///     vec![Text::new("First row, first column"), Text::new("First row, second column")],
///     vec![Text::new("Second row, first column"), Text::new("Second row, second column")]
/// ]);
/// ```
#[allow(missing_debug_implementations)]
pub struct Table<'a, Message, Renderer> {
    rows: Vec<Vec<Element<'a, Message, Renderer>>>,
}

impl<'a, Message, Renderer> Table<'a, Message, Renderer> {
    /// Create a new empty [`Table`].
    ///
    /// [`Table`]: struct.Table.html
    pub fn new() -> Self {
        Self { rows: Vec::new() }
    }

    /// Create a new [`Table`] with the given rows.
    ///
    /// [`Table`]: struct.Table.html
    pub fn from_vec(rows: Vec<Vec<Element<'a, Message, Renderer>>>) -> Self {
        Self { rows }
    }

    /// Adds a row of [`Element`]s to the [`Table`].
    ///
    /// [`Element`]: ../struct.Element.html
    /// [`Table`]: struct.Table.html
    pub fn push(mut self, row: Vec<Element<'a, Message, Renderer>>) -> Self {
        self.rows.push(row);
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Table<'a, Message, Renderer>
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
        let mut layouts = Vec::with_capacity(self.rows.len());
        // store width of each column
        let mut column_widths = Vec::<f32>::new();

        // find out how wide a column is by finding the widest cell in it
        for row in &self.rows {
            let mut layouts_row = Vec::with_capacity(row.len());

            for (index, cell) in row.iter().enumerate() {
                let layout = cell.layout(renderer, limits).size();
                layouts_row.push(layout);

                if let Some(column_width) = column_widths.get_mut(index) {
                    *column_width = column_width.max(layout.width);
                } else {
                    column_widths.insert(index, layout.width);
                }
            }

            layouts.push(layouts_row);
        }

        // list of x alignments for every column
        let column_aligns: Vec<_> = iter::once(&0.)
            .chain(&column_widths)
            .scan(0., |state, width| {
                *state += width;
                Some(*state)
            })
            .collect();

        let mut nodes_table = Vec::with_capacity(self.rows.len());
        let table_width = column_widths.into_iter().sum();
        let mut table_height = 0.;

        for row in layouts {
            let mut nodes_row = Vec::with_capacity(row.len());
            let mut row_height: f32 = 0.;

            for (size, column_align) in row.into_iter().zip(&column_aligns) {
                let node = Node::new(size, Size::new(*column_align, 0.));
                nodes_row.push(node);
                row_height = row_height.max(size.height);
            }

            nodes_table.push(Node::with_children(
                Size::new(table_width, row_height),
                Size::new(0., table_height),
                nodes_row,
            ));

            table_height += row_height;
        }

        Node::with_children(
            Size::new(table_width, table_height),
            Size::ZERO,
            nodes_table,
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        renderer.draw(defaults, layout, cursor_position, &self.rows)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        for row in &self.rows {
            for cell in row {
                cell.hash_layout(state);
            }
        }
    }
}

/// The renderer of a [`Table`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use [`Table`] in your [`UserInterface`].
///
/// [`Table`]: struct.Table.html
/// [renderer]: ../../renderer/index.html
/// [`UserInterface`]: ../../struct.UserInterface.html
pub trait Renderer: crate::Renderer {
    /// Draws a [`Table`].
    ///
    /// It receives:
    /// - the list of rows and it's [`Element`]s
    /// - the [`Layout`] of the rows and their children
    /// - the cursor position
    ///
    /// [`Table`]: struct.Text.html
    /// [`Element`]: ../struct.Element.html
    /// [`Layout`]: ../layout/struct.Layout.html
    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        rows: &[Vec<Element<'_, Message, Self>>],
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<Table<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer,
    Message: 'static,
{
    fn from(
        table: Table<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(table)
    }
}
