//! Control device event delivery.
//!
//! Device events are raw hardware events that are not associated
//! with any particular window.
use crate::Action;
use crate::core::device::Filter;
use crate::task::{self, Task};

/// A device event action.
#[derive(Debug)]
pub enum DeviceAction {
    /// Set the device event filter.
    SetFilter(Filter),
}

/// Sets the device event filter.
///
/// This controls when device events are delivered to subscriptions:
///
/// - [`Filter::Always`]: Device events are always delivered
/// - [`Filter::Never`]: Device events are never delivered
/// - [`Filter::WhenFocused`]: Device events are only delivered when a window has focus (default)
///
/// # Example
///
/// ```no_run
/// use iced::device::{self, Filter};
/// use iced::Task;
///
/// enum Message {
///     // ...
/// }
///
/// fn enable_raw_input() -> Task<Message> {
///     device::set_filter(Filter::Always)
/// }
///
/// fn disable_raw_input() -> Task<Message> {
///     device::set_filter(Filter::Never)
/// }
/// ```
pub fn set_filter<T>(filter: Filter) -> Task<T> {
    task::effect(Action::Device(DeviceAction::SetFilter(filter)))
}
