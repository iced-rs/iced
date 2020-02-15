use crate::{css::Rule, Bus, Css, Element, Widget};

use dodrio::bumpalo;

/// A container that distributes its contents in a grid.
#[allow(missing_debug_implementations)]
pub struct Grid<'a, Message> {
    columns: usize,
    elements: Vec<Element<'a, Message>>,
}

impl<'a, Message> Grid<'a, Message> {
    /// Create a new empty [`Grid`].
    ///
    /// [`Grid`]: struct.Grid.html
    pub fn new(columns: usize) -> Self {
        Self::with_children(columns, Vec::new())
    }

    /// Create a new [`Grid`] with the given [`Element`]s.
    ///
    /// [`Grid`]: struct.Grid.html
    /// [`Element`]: ../struct.Element.html
    pub fn with_children(
        columns: usize,
        elements: Vec<Element<'a, Message>>,
    ) -> Self {
        Self { columns, elements }
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
        let mut style = String::from("grid-template-columns:");

        for _ in 0..self.columns {
            style.push_str(" auto");
        }

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
