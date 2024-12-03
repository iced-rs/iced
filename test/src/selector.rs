use crate::core::text;
use crate::core::widget;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    Id(widget::Id),
    Text(text::Fragment<'static>),
}

impl From<widget::Id> for Selector {
    fn from(id: widget::Id) -> Self {
        Self::Id(id)
    }
}

impl From<&'static str> for Selector {
    fn from(id: &'static str) -> Self {
        Self::Id(widget::Id::new(id))
    }
}

pub fn text(fragment: impl text::IntoFragment<'static>) -> Selector {
    Selector::Text(fragment.into_fragment())
}
