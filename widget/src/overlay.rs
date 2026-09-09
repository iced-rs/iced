//! Display interactive elements on top of other widgets.
pub mod menu;
/// The [`Position`] of a popup (a popover or a tooltip) and the logic to
/// resolve it into a concrete [`crate::core::Rectangle`].
pub mod position;

pub use position::Position;
