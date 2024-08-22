use crate::container;
use crate::core::{self, Element};

/// The controls of a [`Pane`].
///
/// [`Pane`]: super::Pane
#[allow(missing_debug_implementations)]
pub struct Controls<
    'a,
    Message,
    Theme = crate::Theme,
    Renderer = crate::Renderer,
> where
    Theme: container::Catalog,
    Renderer: core::Renderer,
{
    pub(super) full: Element<'a, Message, Theme, Renderer>,
    pub(super) compact: Option<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> Controls<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog,
    Renderer: core::Renderer,
{
    /// Creates a new [`Controls`] with the given content.
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            full: content.into(),
            compact: None,
        }
    }

    /// Creates a new [`Controls`] with a full and compact variant.
    /// If there is not enough room to show the full variant without overlap,
    /// then the compact variant will be shown instead.
    pub fn dynamic(
        full: impl Into<Element<'a, Message, Theme, Renderer>>,
        compact: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            full: full.into(),
            compact: Some(compact.into()),
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Element<'a, Message, Theme, Renderer>>
    for Controls<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog,
    Renderer: core::Renderer,
{
    fn from(value: Element<'a, Message, Theme, Renderer>) -> Self {
        Self::new(value)
    }
}
