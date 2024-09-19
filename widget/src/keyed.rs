//! Keyed widgets can provide hints to ensure continuity.
//!
//! # What is continuity?
//! Continuity is the feeling of persistence of state.
//!
//! In a graphical user interface, users expect widgets to have a
//! certain degree of continuous state. For instance, a text input
//! that is focused should stay focused even if the widget tree
//! changes slightly.
//!
//! Continuity is tricky in `iced` and the Elm Architecture because
//! the whole widget tree is rebuilt during every `view` call. This is
//! very convenient from a developer perspective because you can build
//! extremely dynamic interfaces without worrying about changing state.
//!
//! However, the tradeoff is that determining what changed becomes hard
//! for `iced`. If you have a list of things, adding an element at the
//! top may cause a loss of continuity on every element on the list!
//!
//! # How can we keep continuity?
//! The good news is that user interfaces generally have a static widget
//! structure. This structure can be relied on to ensure some degree of
//! continuity. `iced` already does this.
//!
//! However, sometimes you have a certain part of your interface that is
//! quite dynamic. For instance, a list of things where items may be added
//! or removed at any place.
//!
//! There are different ways to mitigate this during the reconciliation
//! stage, but they involve comparing trees at certain depths and
//! backtracking... Quite computationally expensive.
//!
//! One approach that is cheaper consists in letting the user provide some hints
//! about the identities of the different widgets so that they can be compared
//! directly without going deeper.
//!
//! The widgets in this module will all ask for a "hint" of some sort. In order
//! to help them keep continuity, you need to make sure the hint stays the same
//! for the same items in your user interface between `view` calls.
pub mod column;

pub use column::Column;

/// Creates a keyed [`Column`] with the given children.
///
/// Keyed columns distribute content vertically while keeping continuity.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::keyed_column;
///
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     keyed_column![
///         (0, "Item 0"),
///         (1, "Item 1"),
///         (2, "Item 2"),
///     ].into()
/// }
/// ```
#[macro_export]
macro_rules! keyed_column {
    () => (
        $crate::keyed::Column::new()
    );
    ($(($key:expr, $x:expr)),+ $(,)?) => (
        $crate::keyed::Column::with_children(vec![$(($key, $crate::core::Element::from($x))),+])
    );
}
