#![allow(missing_docs, missing_debug_implementations)]
use crate::core;
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget;
use crate::core::{
    Background, Element, Layout, Length, Pixels, Rectangle, Size, Widget,
};

pub fn table<'a, R, T, Message, Theme, Renderer>(
    columns: impl IntoIterator<Item = Column<'a, T, Message, Theme, Renderer>>,
    rows: R,
) -> Table<'a, Message, Theme, Renderer>
where
    R: IntoIterator<Item = T>,
    R::IntoIter: Clone,
    Theme: Catalog,
    Renderer: core::Renderer,
{
    Table::new(columns, rows)
}

pub fn column<'a, T, E, Message, Theme, Renderer>(
    header: impl Into<Element<'a, Message, Theme, Renderer>>,
    view: impl Fn(T) -> E + 'a,
) -> Column<'a, T, Message, Theme, Renderer>
where
    T: 'a,
    E: Into<Element<'a, Message, Theme, Renderer>>,
{
    Column {
        header: header.into(),
        view: Box::new(move |data| view(data).into()),
        width: Length::Shrink,
    }
}

pub struct Table<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Theme: Catalog,
{
    columns: Vec<Length>,
    cells: Vec<Element<'a, Message, Theme, Renderer>>,
    width: Length,
    height: Length,
    padding_x: f32,
    padding_y: f32,
    separator_x: f32,
    separator_y: f32,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> Table<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: core::Renderer,
{
    pub fn new<R, T>(
        columns: impl IntoIterator<Item = Column<'a, T, Message, Theme, Renderer>>,
        rows: R,
    ) -> Self
    where
        R: IntoIterator<Item = T>,
        R::IntoIter: Clone,
    {
        let columns = columns.into_iter();
        let rows = rows.into_iter();

        let mut width = Length::Shrink;
        let mut height = Length::Shrink;
        let mut cells = Vec::with_capacity(
            columns.size_hint().0 * (1 + rows.size_hint().0),
        );

        Self {
            columns: columns
                .into_iter()
                .map(|column| {
                    let mut column_width = column.width;

                    cells.push(column.header);
                    cells.extend(rows.clone().map(|row| {
                        let cell = (column.view)(row);
                        let size_hint = cell.as_widget().size_hint();

                        column_width = column_width.enclose(size_hint.width);
                        height = height.enclose(size_hint.height);

                        cell
                    }));

                    width = width.enclose(column_width);

                    column_width
                })
                .collect(),
            cells,
            width,
            height,
            padding_x: 10.0,
            padding_y: 10.0,
            separator_x: 1.0,
            separator_y: 1.0,
            class: Theme::default(),
        }
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn padding(self, padding: impl Into<Pixels>) -> Self {
        let padding = padding.into();

        self.padding_x(padding).padding_y(padding)
    }

    pub fn padding_x(mut self, padding: impl Into<Pixels>) -> Self {
        self.padding_x = padding.into().0;
        self
    }

    pub fn padding_y(mut self, padding: impl Into<Pixels>) -> Self {
        self.padding_y = padding.into().0;
        self
    }

    pub fn separator(self, separator: impl Into<Pixels>) -> Self {
        let separator = separator.into();

        self.separator_x(separator).separator_y(separator)
    }

    pub fn separator_x(mut self, separator: impl Into<Pixels>) -> Self {
        self.separator_x = separator.into().0;
        self
    }

    pub fn separator_y(mut self, separator: impl Into<Pixels>) -> Self {
        self.separator_y = separator.into().0;
        self
    }
}

pub struct Metrics {
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
        let limits = limits.width(self.width).height(self.height);
        let rows = self.cells.len() / self.columns.len();
        let available = limits.max();

        let mut cells = Vec::with_capacity(self.cells.len());
        cells.resize(self.cells.len(), layout::Node::default());

        metrics.columns = vec![0.0; self.columns.len()];
        metrics.rows = vec![0.0; rows];

        let mut column_factors = vec![0; self.columns.len()];
        let mut row_factors = vec![0; rows];

        let spacing_x = self.padding_x * 2.0 + self.separator_x;
        let spacing_y = self.padding_y * 2.0 + self.separator_y;

        // FIRST PASS
        // Lay out non-fluid cells
        let mut x = self.padding_x;
        let mut y = self.padding_y;

        for (i, (cell, state)) in
            self.cells.iter().zip(&mut tree.children).enumerate()
        {
            let column = i / rows;
            let row = i % rows;
            let size = cell.as_widget().size();

            if size.width.fill_factor() != 0 || size.height.fill_factor() != 0 {
                column_factors[column] =
                    column_factors[column].max(size.width.fill_factor());

                row_factors[row] =
                    row_factors[row].max(size.height.fill_factor());

                continue;
            }

            let limits = layout::Limits::new(
                Size::ZERO,
                Size::new(available.width - x, available.height - y),
            )
            .width(self.columns[i / rows]);

            let layout = cell.as_widget().layout(state, renderer, &limits);
            let size = layout.size();

            metrics.columns[column] = metrics.columns[column].max(size.width);
            metrics.rows[row] = metrics.rows[row].max(size.height);
            cells[i] = layout;

            if row == 0 {
                y = self.padding_y;

                if column > 0 {
                    x += metrics.columns[column - 1] + spacing_x;
                }
            } else {
                y += size.height + spacing_y;
            }
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
            available.height
                - metrics
                    .rows
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| row_factors[*i] == 0)
                    .map(|(_, height)| height)
                    .sum::<f32>(),
        );

        let width_unit = (left.width
            - spacing_x * self.columns.len().saturating_sub(1) as f32
            - self.padding_x * 2.0)
            / column_factors.iter().sum::<u16>() as f32;

        let height_unit = (left.height
            - spacing_y * rows.saturating_sub(1) as f32
            - self.padding_y * 2.0)
            / row_factors.iter().sum::<u16>() as f32;

        let mut x = self.padding_x;
        let mut y = self.padding_y;

        for (i, (cell, state)) in
            self.cells.iter().zip(&mut tree.children).enumerate()
        {
            let column = i / rows;
            let row = i % rows;
            let size = cell.as_widget().size();

            if size.width.fill_factor() != 0 || size.height.fill_factor() != 0 {
                let column_factor = column_factors[column];
                let row_factor = row_factors[row];

                let max_width = if column_factor == 0 {
                    (available.width - x).max(0.0)
                } else {
                    width_unit * column_factor as f32
                };

                let max_height = if row_factor == 0 {
                    (available.height - y).max(0.0)
                } else {
                    height_unit * row_factor as f32
                };

                let limits = layout::Limits::new(
                    Size::ZERO,
                    Size::new(max_width, max_height),
                )
                .width(self.columns[i / rows]);

                let layout = cell.as_widget().layout(state, renderer, &limits);
                let size = layout.size();

                metrics.columns[column] =
                    metrics.columns[column].max(size.width);
                metrics.rows[row] = metrics.rows[row].max(size.height);
                cells[i] = layout;
            }

            if row == 0 {
                y = self.padding_y;

                if column > 0 {
                    x += metrics.columns[column - 1] + spacing_x;
                }
            } else {
                y += cells[i].size().height + spacing_y;
            }
        }

        // THIRD PASS
        // Position each cell
        let mut x = self.padding_x;
        let mut y = self.padding_y;

        for (i, cell) in cells.iter_mut().enumerate() {
            let column = i / rows;
            let row = i % rows;

            if row == 0 {
                y = self.padding_y;

                if column > 0 {
                    x += metrics.columns[column - 1] + spacing_x;
                }
            }

            cell.move_to_mut((x, y));

            y += metrics.rows[row] + spacing_y;
        }

        let intrinsic = limits.resolve(
            self.width,
            self.height,
            Size::new(
                x + metrics
                    .columns
                    .last()
                    .copied()
                    .map(|width| width + self.padding_x)
                    .unwrap_or_default(),
                y - spacing_y + self.padding_y,
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

pub struct Column<
    'a,
    T,
    Message,
    Theme = crate::Theme,
    Renderer = crate::Renderer,
> {
    header: Element<'a, Message, Theme, Renderer>,
    view: Box<dyn Fn(T) -> Element<'a, Message, Theme, Renderer> + 'a>,
    width: Length,
}

impl<'a, T, Message, Theme, Renderer> Column<'a, T, Message, Theme, Renderer> {
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub separator_x: Background,
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

    Style {
        separator_x: palette.background.strong.color.into(),
        separator_y: palette.background.strong.color.into(),
    }
}
