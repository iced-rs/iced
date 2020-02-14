use crate::{Bus, Css, Element, Widget};

use dodrio::bumpalo;

/// A table that distributes its contents horizontally and vertically.
#[allow(missing_debug_implementations)]
pub struct Table<'a, Message> {
    rows: Vec<Vec<Element<'a, Message>>>,
}

impl<'a, Message> Table<'a, Message> {
    /// Create a new empty [`Table`].
    ///
    /// [`Table`]: struct.Table.html
    pub fn new() -> Self {
        Self { rows: Vec::new() }
    }

    /// Create a new [`Table`] with the given rows.
    ///
    /// [`Table`]: struct.Table.html
    pub fn from_vec(rows: Vec<Vec<Element<'a, Message>>>) -> Self {
        Self { rows }
    }

    /// Adds a row of [`Element`]s to the [`Table`].
    ///
    /// [`Element`]: ../struct.Element.html
    /// [`Table`]: struct.Table.html
    pub fn push(mut self, row: Vec<Element<'a, Message>>) -> Self {
        self.rows.push(row);
        self
    }
}

impl<'a, Message> Widget<Message> for Table<'a, Message> {
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        publish: &Bus<Message>,
        style_sheet: &mut Css<'b>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;

        let mut table = table(bump);

        for row in &self.rows {
            let mut tr = tr(bump);

            for cell in row {
                tr = tr.child(
                    td(bump)
                        .child(cell.node(bump, publish, style_sheet))
                        .finish(),
                )
            }

            table = table.child(tr.finish());
        }

        table.finish()
    }
}

impl<'a, Message> From<Table<'a, Message>> for Element<'a, Message>
where
    Message: 'static,
{
    fn from(table: Table<'a, Message>) -> Element<'a, Message> {
        Element::new(table)
    }
}
