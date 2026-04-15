//! Focus change events.

/// A focus-related event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// The focused widget changed.
    ///
    /// Produced by the runtime after a widget operation modifies focus
    /// (e.g. `focus_next`, `focus_directional`, `focus(id)`, `unfocus`,
    /// or `auto_focus`). Mouse-click focus changes in widgets that
    /// participate in the focus system also produce this event.
    FocusChanged,
}
