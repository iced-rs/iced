use std::borrow::Cow;

use crate::A11yId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Description<'a> {
    Text(Cow<'a, str>),
    Id(Vec<A11yId>),
}

// Describes a widget
pub trait Describes {
    fn description(&self) -> Vec<A11yId>;
}

// Labels a widget
pub trait Labels {
    fn label(&self) -> Vec<A11yId>;
}
