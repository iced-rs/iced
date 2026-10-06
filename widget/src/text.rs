//! Draw and interact with text.
mod rich;

pub use crate::core::text::highlighter;
pub use crate::core::text::{Fragment, Highlighter, IntoFragment, Parser, Span};
pub use crate::core::widget::text::*;
pub use rich::Rich;

/// A bunch of text.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
/// # use iced::widget::Widget;
/// # pub type State = ();
/// use iced::widget::text;
/// use iced::color;
///
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> impl Widget<Message> {
///     text("Hello, this is iced!")
///         .size(20)
///         .color(color!(0x0000ff))
/// }
/// ```
pub type Text<'a, Theme = crate::Theme> = crate::core::widget::Text<'a, Theme>;
