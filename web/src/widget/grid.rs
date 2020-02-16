use crate::{css::Rule, Bus, Css, Element, Widget};

use dodrio::bumpalo;

/// A container that distributes its contents in a grid.
#[allow(missing_debug_implementations)]
pub struct Grid<'a, Message> {
    strategy: Strategy,
    elements: Vec<Element<'a, Message>>,
}

enum Strategy {
    Columns(usize),
    ColumnWidth(u16),
}

impl<'a, Message> Grid<'a, Message> {
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
        E: Into<Element<'a, Message>>,
    {
        self.elements.push(element.into());
        self
    }
}

impl<'a, Message> Widget<Message> for Grid<'a, Message> {
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        publish: &Bus<Message>,
        style_sheet: &mut Css<'b>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::div;

        let mut grid = div(bump);

        match self.strategy {
            Strategy::Columns(columns) => {
                grid = grid.attr(
                    "class",
                    bumpalo::format!(in bump, "{}", style_sheet.insert(bump, Rule::Grid))
                        .into_bump_str()
                );

                if columns > 0 {
                    let mut style = String::from("grid-template-columns:");

                    for _ in 0..columns {
                        style.push_str(" auto");
                    }

                    grid = grid.attr(
                        "style",
                        bumpalo::format!(in bump, "{}", style).into_bump_str(),
                    );
                }
            }
            Strategy::ColumnWidth(column_width) => {
                grid = grid.attr(
                    "class",
                    bumpalo::format!(in bump, "{}", style_sheet.insert(bump, Rule::Flex(column_width)))
                        .into_bump_str()
                );
            }
        }

        for element in &self.elements {
            grid = grid.child(element.node(bump, publish, style_sheet));
        }

        grid.finish()
    }
}

impl<'a, Message> From<Grid<'a, Message>> for Element<'a, Message>
where
    Message: 'static,
{
    fn from(grid: Grid<'a, Message>) -> Element<'a, Message> {
        Element::new(grid)
    }
}
