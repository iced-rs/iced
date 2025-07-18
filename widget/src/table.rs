//! Display tables.
use crate::core;
use crate::core::alignment;
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget;
use crate::core::{
    Alignment, Background, Element, Layout, Length, Pixels, Rectangle, Size,
    Widget,
};

/// Creates a new [`Table`] with the given columns and rows.
///
/// Columns can be created using the [`column()`] function, while rows can be any
/// iterator over some data type `T`.
pub fn table<'a, 'b, T, Message, Theme, Renderer>(
    columns: impl IntoIterator<Item = Column<'a, 'b, T, Message, Theme, Renderer>>,
    rows: impl IntoIterator<Item = T>,
) -> Table<'a, Message, Theme, Renderer>
where
    T: Clone,
    Theme: Catalog,
    Renderer: core::Renderer,
{
    Table::new(columns, rows)
}

/// Creates a new [`Column`] with the given header and view function.
///
/// The view function will be called for each row in a [`Table`] and it must
/// produce the resulting contents of a cell.
pub fn column<'a, 'b, T, E, Message, Theme, Renderer>(
    header: impl Into<Element<'a, Message, Theme, Renderer>>,
    view: impl Fn(T) -> E + 'b,
) -> Column<'a, 'b, T, Message, Theme, Renderer>
where
    T: 'a,
    E: Into<Element<'a, Message, Theme, Renderer>>,
{
    Column {
        header: header.into(),
        view: Box::new(move |data| view(data).into()),
        width: Length::Shrink,
        align_x: alignment::Horizontal::Left,
        align_y: alignment::Vertical::Top,
    }
}

/// A grid-like visual representation of data distributed in columns and rows.
#[allow(missing_debug_implementations)]
pub struct Table<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Theme: Catalog,
{
    columns: Vec<Column_>,
    cells: Vec<Element<'a, Message, Theme, Renderer>>,
    width: Length,
    height: Length,
    padding_x: f32,
    padding_y: f32,
    separator_x: f32,
    separator_y: f32,
    class: Theme::Class<'a>,
}

struct Column_ {
    width: Length,
    align_x: alignment::Horizontal,
    align_y: alignment::Vertical,
}

impl<'a, Message, Theme, Renderer> Table<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: core::Renderer,
{
    /// Creates a new [`Table`] with the given columns and rows.
    ///
    /// Columns can be created using the [`column()`] function, while rows can be any
    /// iterator over some data type `T`.
    pub fn new<'b, T>(
        columns: impl IntoIterator<
            Item = Column<'a, 'b, T, Message, Theme, Renderer>,
        >,
        rows: impl IntoIterator<Item = T>,
    ) -> Self
    where
        T: Clone,
    {
        let columns = columns.into_iter();
        let rows = rows.into_iter();

        let mut width = Length::Shrink;
        let mut height = Length::Shrink;

        let mut cells = Vec::with_capacity(
            columns.size_hint().0 * (1 + rows.size_hint().0),
        );

        let (mut columns, views): (Vec<_>, Vec<_>) = columns
            .map(|column| {
                width = width.enclose(column.width);

                cells.push(column.header);

                (
                    Column_ {
                        width: column.width,
                        align_x: column.align_x,
                        align_y: column.align_y,
                    },
                    column.view,
                )
            })
            .collect();

        for row in rows {
            for view in &views {
                let cell = view(row.clone());
                let size_hint = cell.as_widget().size_hint();

                height = height.enclose(size_hint.height);

                cells.push(cell);
            }
        }

        if width == Length::Shrink {
            if let Some(first) = columns.first_mut() {
                first.width = Length::Fill;
            }
        }

        Self {
            columns,
            cells,
            width,
            height,
            padding_x: 10.0,
            padding_y: 5.0,
            separator_x: 1.0,
            separator_y: 1.0,
            class: Theme::default(),
        }
    }

    /// Sets the width of the [`Table`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the padding of the cells of the [`Table`].
    pub fn padding(self, padding: impl Into<Pixels>) -> Self {
        let padding = padding.into();

        self.padding_x(padding).padding_y(padding)
    }

    /// Sets the horizontal padding of the cells of the [`Table`].
    pub fn padding_x(mut self, padding: impl Into<Pixels>) -> Self {
        self.padding_x = padding.into().0;
        self
    }

    /// Sets the vertical padding of the cells of the [`Table`].
    pub fn padding_y(mut self, padding: impl Into<Pixels>) -> Self {
        self.padding_y = padding.into().0;
        self
    }

    /// Sets the thickness of the line separator between the cells of the [`Table`].
    pub fn separator(self, separator: impl Into<Pixels>) -> Self {
        let separator = separator.into();

        self.separator_x(separator).separator_y(separator)
    }

    /// Sets the thickness of the horizontal line separator between the cells of the [`Table`].
    pub fn separator_x(mut self, separator: impl Into<Pixels>) -> Self {
        self.separator_x = separator.into().0;
        self
    }

    /// Sets the thickness of the vertical line separator between the cells of the [`Table`].
    pub fn separator_y(mut self, separator: impl Into<Pixels>) -> Self {
        self.separator_y = separator.into().0;
        self
    }
}

struct Metrics {
    columns: Vec<f32>,
    rows: Vec<f32>,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Table<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: core::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<Metrics>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(Metrics {
            columns: Vec::new(),
            rows: Vec::new(),
        })
    }

    fn children(&self) -> Vec<widget::Tree> {
        self.cells
            .iter()
            .map(|cell| widget::Tree::new(cell.as_widget()))
            .collect()
    }

    fn diff(&self, state: &mut widget::Tree) {
        state.diff_children(&self.cells);
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let metrics = tree.state.downcast_mut::<Metrics>();
        let columns = self.columns.len();
        let rows = self.cells.len() / columns;

        let limits = limits.width(self.width).height(self.height);
        let available = limits.max();
        let table_fluid = self.width.fluid();

        let mut cells = Vec::with_capacity(self.cells.len());
        cells.resize(self.cells.len(), layout::Node::default());

        metrics.columns = vec![0.0; self.columns.len()];
        metrics.rows = vec![0.0; rows];

        let mut column_factors = vec![0; self.columns.len()];
        let mut total_row_factors = 0;
        let mut total_fluid_height = 0.0;
        let mut row_factor = 0;

        let spacing_x = self.padding_x * 2.0 + self.separator_x;
        let spacing_y = self.padding_y * 2.0 + self.separator_y;

        // FIRST PASS
        // Lay out non-fluid cells
        let mut x = self.padding_x;
        let mut y = self.padding_y;

        for (i, (cell, state)) in
            self.cells.iter().zip(&mut tree.children).enumerate()
        {
            let row = i / columns;
            let column = i % columns;

            let width = self.columns[column].width;
            let size = cell.as_widget().size();

            if column == 0 {
                x = self.padding_x;

                if row > 0 {
                    y += metrics.rows[row - 1] + spacing_y;

                    if row_factor != 0 {
                        total_fluid_height += metrics.rows[row - 1];
                        total_row_factors += row_factor;

                        row_factor = 0;
                    }
                }
            }

            let width_factor = width.fill_factor();
            let height_factor = size.height.fill_factor();

            if width_factor != 0 || height_factor != 0 || size.width.is_fill() {
                column_factors[column] =
                    column_factors[column].max(width_factor);

                row_factor = row_factor.max(height_factor);

                continue;
            }

            let limits = layout::Limits::new(
                Size::ZERO,
                Size::new(available.width - x, available.height - y),
            )
            .width(width);

            let layout = cell.as_widget().layout(state, renderer, &limits);
            let size = limits.resolve(width, Length::Shrink, layout.size());

            metrics.columns[column] = metrics.columns[column].max(size.width);
            metrics.rows[row] = metrics.rows[row].max(size.height);
            cells[i] = layout;

            x += size.width + spacing_x;
        }

        // SECOND PASS
        // Lay out fluid cells, using metrics from the first pass as limits
        let left = Size::new(
            available.width
                - metrics
                    .columns
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| column_factors[*i] == 0)
                    .map(|(_, width)| width)
                    .sum::<f32>(),
            available.height - total_fluid_height,
        );

        let width_unit = (left.width
            - spacing_x * self.columns.len().saturating_sub(1) as f32
            - self.padding_x * 2.0)
            / column_factors.iter().sum::<u16>() as f32;

        let height_unit = (left.height
            - spacing_y * rows.saturating_sub(1) as f32
            - self.padding_y * 2.0)
            / total_row_factors as f32;

        let mut x = self.padding_x;
        let mut y = self.padding_y;

        for (i, (cell, state)) in
            self.cells.iter().zip(&mut tree.children).enumerate()
        {
            let row = i / columns;
            let column = i % columns;

            let size = cell.as_widget().size();

            let width = self.columns[column].width;
            let width_factor = width.fill_factor();
            let height_factor = size.height.fill_factor();

            if column == 0 {
                x = self.padding_x;

                if row > 0 {
                    y += metrics.rows[row - 1] + spacing_y;
                }
            }

            if width_factor == 0
                && size.width.fill_factor() == 0
                && size.height.fill_factor() == 0
            {
                continue;
            }

            let max_width = if width_factor == 0 {
                if size.width.is_fill() {
                    metrics.columns[column]
                } else {
                    (available.width - x).max(0.0)
                }
            } else {
                width_unit * width_factor as f32
            };

            let max_height = if height_factor == 0 {
                if size.height.is_fill() {
                    metrics.rows[row]
                } else {
                    (available.height - y).max(0.0)
                }
            } else {
                height_unit * height_factor as f32
            };

            let limits = layout::Limits::new(
                Size::ZERO,
                Size::new(max_width, max_height),
            )
            .width(width);

            let layout = cell.as_widget().layout(state, renderer, &limits);
            let size = limits.resolve(
                if let Length::Fixed(_) = width {
                    width
                } else {
                    table_fluid
                },
                Length::Shrink,
                layout.size(),
            );

            metrics.columns[column] = metrics.columns[column].max(size.width);
            metrics.rows[row] = metrics.rows[row].max(size.height);
            cells[i] = layout;

            x += size.width + spacing_x;
        }

        // THIRD PASS
        // Position each cell
        let mut x = self.padding_x;
        let mut y = self.padding_y;

        for (i, cell) in cells.iter_mut().enumerate() {
            let row = i / columns;
            let column = i % columns;

            if column == 0 {
                x = self.padding_x;

                if row > 0 {
                    y += metrics.rows[row - 1] + spacing_y;
                }
            }

            let Column_ {
                align_x, align_y, ..
            } = &self.columns[column];

            cell.move_to_mut((x, y));
            cell.align_mut(
                Alignment::from(*align_x),
                Alignment::from(*align_y),
                Size::new(metrics.columns[column], metrics.rows[row]),
            );

            x += metrics.columns[column] + spacing_x;
        }

        let intrinsic = limits.resolve(
            self.width,
            self.height,
            Size::new(
                x - spacing_x + self.padding_x,
                y + metrics
                    .rows
                    .last()
                    .copied()
                    .map(|height| height + self.padding_y)
                    .unwrap_or_default(),
            ),
        );

        layout::Node::with_children(intrinsic, cells)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for ((cell, state), layout) in
            self.cells.iter().zip(&tree.children).zip(layout.children())
        {
            cell.as_widget()
                .draw(state, renderer, theme, style, layout, cursor, viewport);
        }

        let bounds = layout.bounds();
        let metrics = tree.state.downcast_ref::<Metrics>();
        let style = theme.style(&self.class);

        if self.separator_x > 0.0 {
            let mut x = self.padding_x;

            for width in
                &metrics.columns[..metrics.columns.len().saturating_sub(1)]
            {
                x += width + self.padding_x;

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x + x,
                            y: bounds.y,
                            width: self.separator_x,
                            height: bounds.height,
                        },
                        snap: true,
                        ..renderer::Quad::default()
                    },
                    style.separator_x,
                );

                x += self.separator_x + self.padding_x;
            }
        }

        if self.separator_y > 0.0 {
            let mut y = self.padding_y;

            for height in &metrics.rows[..metrics.rows.len().saturating_sub(1)]
            {
                y += height + self.padding_y;

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x,
                            y: bounds.y + y,
                            width: bounds.width,
                            height: self.separator_y,
                        },
                        snap: true,
                        ..renderer::Quad::default()
                    },
                    style.separator_y,
                );

                y += self.separator_y + self.padding_y;
            }
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Table<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::Renderer + 'a,
{
    fn from(table: Table<'a, Message, Theme, Renderer>) -> Self {
        Element::new(table)
    }
}

/// A vertical visualization of some data with a header.
#[allow(missing_debug_implementations)]
pub struct Column<
    'a,
    'b,
    T,
    Message,
    Theme = crate::Theme,
    Renderer = crate::Renderer,
> {
    header: Element<'a, Message, Theme, Renderer>,
    view: Box<dyn Fn(T) -> Element<'a, Message, Theme, Renderer> + 'b>,
    width: Length,
    align_x: alignment::Horizontal,
    align_y: alignment::Vertical,
}

impl<'a, 'b, T, Message, Theme, Renderer>
    Column<'a, 'b, T, Message, Theme, Renderer>
{
    /// Sets the width of the [`Column`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the alignment for the horizontal axis of the [`Column`].
    pub fn align_x(
        mut self,
        alignment: impl Into<alignment::Horizontal>,
    ) -> Self {
        self.align_x = alignment.into();
        self
    }

    /// Sets the alignment for the vertical axis of the [`Column`].
    pub fn align_y(
        mut self,
        alignment: impl Into<alignment::Vertical>,
    ) -> Self {
        self.align_y = alignment.into();
        self
    }
}

/// The appearance of a [`Table`].
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// The background color of the horizontal line separator between cells.
    pub separator_x: Background,
    /// The background color of the vertical line separator between cells.
    pub separator_y: Background,
}

/// The theme catalog of a [`Table`].
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// A styling function for a [`Table`].
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl<Theme> From<Style> for StyleFn<'_, Theme> {
    fn from(style: Style) -> Self {
        Box::new(move |_theme| style)
    }
}

impl Catalog for crate::Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

/// The default style of a [`Table`].
pub fn default(theme: &crate::Theme) -> Style {
    let palette = theme.extended_palette();
    let separator = palette.background.strong.color.into();

    Style {
        separator_x: separator,
        separator_y: separator,
    }
}
