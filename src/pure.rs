//! Leverage pure, virtual widgets in your application.
//!
//! The widgets found in this module are completely stateless versions of
//! [the original widgets].
//!
//! Effectively, this means that, as a user of the library, you do not need to
//! keep track of the local state of each widget (e.g. [`button::State`]).
//! Instead, the runtime will keep track of everything for you!
//!
//! You can embed pure widgets anywhere in your [impure `Application`] using the
//! [`Pure`] widget and some [`State`].
//!
//! In case you want to only use pure widgets in your application, this module
//! offers an alternate [`Application`] trait with a completely pure `view`
//! method.
//!
//! [the original widgets]: crate::widget
//! [`button::State`]: crate::widget::button::State
//! [impure `Application`]: crate::Application
pub use iced_pure::{
    Button as _, Column as _, Element as _, Image as _, Row as _, Text as _, *,
};

/// A generic, pure [`Widget`].
pub type Element<'a, Message> =
    iced_pure::Element<'a, Message, crate::Renderer>;

/// A pure container widget.
pub type Container<'a, Message> =
    iced_pure::Container<'a, Message, crate::Renderer>;

/// A pure column widget.
pub type Column<'a, Message> = iced_pure::Column<'a, Message, crate::Renderer>;

/// A pure row widget.
pub type Row<'a, Message> = iced_pure::Row<'a, Message, crate::Renderer>;

/// A pure button widget.
pub type Button<'a, Message> = iced_pure::Button<'a, Message, crate::Renderer>;

/// A pure text widget.
pub type Text = iced_pure::Text<crate::Renderer>;

#[cfg(feature = "image")]
/// A pure image widget.
pub type Image = iced_pure::Image<crate::widget::image::Handle>;

mod application;
mod sandbox;

pub use application::Application;
pub use sandbox::Sandbox;

#[cfg(feature = "canvas")]
pub use iced_graphics::widget::pure::canvas;

#[cfg(feature = "canvas")]
pub use canvas::Canvas;
