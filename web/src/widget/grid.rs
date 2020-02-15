use crate::{css::Rule, Bus, Css, Element, Widget};

use dodrio::bumpalo;

/// A container that distributes its contents in a grid.
#[allow(missing_debug_implementations)]
pub struct Grid<'a, Message> {
    columns: Option<usize>,
    column_width: Option<u16>,
    elements: Vec<Element<'a, Message>>,
}

impl<'a, Message> Grid<'a, Message> {
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
    pub fn with_children(elements: Vec<Element<'a, Message>>) -> Self {
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
        self.columns = Some(columns);
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
        use dodrio::builder::*;

        let mut grid = div(bump);

        let column_width = if let Some(column_width) = self.column_width {
            format!("{}px", column_width)
        } else {
            String::from("auto")
        };

        let style = if let Some(columns) = self.columns {
            let mut style = String::from("grid-template-columns:");

            for _ in 0..columns {
                style.push_str(" ");
                style.push_str(&column_width);
            }

            style
        } else if let Some(column_width) = self.column_width {
            format!("width: 100%; grid-template-columns: repeat(auto-fit, minmax({}px, 1fr))", column_width)
        } else {
            format!(
                "grid-template-columns: repeat({}, 1fr)",
                self.elements.len()
            )
        };

        grid = grid.attr(
            "class",
            bumpalo::format!(in bump, "{}", style_sheet.insert(bump, Rule::Grid))
                .into_bump_str(),
        ).attr(
            "style",
            bumpalo::format!(in bump, "{}", style).into_bump_str(),
        );

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
