//! Display a horizontal or vertical rule for dividing content.

use crate::{Backend, Renderer};
use iced_native::rule;

pub use iced_style::rule::{FillMode, Style, StyleSheet};

/// Display a horizontal or vertical rule for dividing content.
///
/// This is an alias of an `iced_native` rule with an `iced_graphics::Renderer`.
pub type Rule<Backend> = iced_native::Rule<Renderer<Backend>>;

impl<B> rule::Renderer for Renderer<B>
where
    B: Backend,
{
    type Style = Box<dyn StyleSheet>;
}
