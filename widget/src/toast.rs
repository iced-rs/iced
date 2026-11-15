//! Toast notifications.
//!
//! A [`Toast`] is a small, self-contained message with an optional close
//! button. It is meant to be displayed on top of the rest of your interface,
//! usually with [`stack`](crate::stack).
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
//! # use iced::widget::Widget;
//! use iced::widget::{stack, toast};
//!
//! enum Message {
//!     Close,
//! }
//!
//! fn view() -> impl Widget<Message> {
//!     stack![
//!         "Your interface",
//!         toast::Toast::new("Saved", "Your changes were saved.")
//!             .status(toast::Status::Success)
//!             .on_close(Message::Close)
//!             .view()
//!     ]
//! }
//! ```
use std::borrow::Cow;
use std::fmt;

use crate::core::{Element, Length, Padding, Widget};
use crate::{Container, Text, button, container, rule, space};

/// The default duration, in seconds, for which a toast should be shown.
pub const DEFAULT_TIMEOUT: u64 = 5;

/// The status of a [`Toast`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Status {
    /// A primary status.
    #[default]
    Primary,
    /// A secondary status.
    Secondary,
    /// A success status.
    Success,
    /// A danger status.
    Danger,
    /// A warning status.
    Warning,
}

impl Status {
    /// Every [`Status`].
    pub const ALL: &'static [Self] = &[
        Self::Primary,
        Self::Secondary,
        Self::Success,
        Self::Danger,
        Self::Warning,
    ];

    fn style(self, theme: &crate::Theme) -> container::Style {
        match self {
            Self::Primary => container::primary(theme),
            Self::Secondary => container::secondary(theme),
            Self::Success => container::success(theme),
            Self::Danger => container::danger(theme),
            Self::Warning => container::warning(theme),
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
            Self::Success => "Success",
            Self::Danger => "Danger",
            Self::Warning => "Warning",
        }
        .fmt(f)
    }
}

/// A toast notification.
///
/// A [`Toast`] is a builder. Call [`Toast::view`] to turn it into an
/// [`Element`] that can be displayed in your interface.
pub struct Toast<'a, Message> {
    title: Cow<'a, str>,
    body: Cow<'a, str>,
    status: Status,
    on_close: Option<Message>,
    width: Length,
    padding: Padding,
}

impl<'a, Message> Toast<'a, Message> {
    /// Creates a new [`Toast`] with the given title and body.
    pub fn new(title: impl Into<Cow<'a, str>>, body: impl Into<Cow<'a, str>>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            status: Status::Primary,
            on_close: None,
            width: Length::Fill,
            padding: 0.into(),
        }
    }

    /// Sets the [`Status`] of the [`Toast`].
    pub fn status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }

    /// Sets the message produced when the close button is pressed.
    ///
    /// If no message is set, no close button is shown.
    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    /// Sets the width of the [`Toast`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the padding of the [`Toast`].
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Turns the [`Toast`] into an [`Element`].
    pub fn view<Renderer>(mut self) -> Element<'a, Message, crate::Theme, Renderer>
    where
        Renderer: crate::core::Renderer + crate::core::text::Renderer + 'a,
        Message: Clone + 'a,
    {
        let close = button("X").padding(5).on_press_maybe(self.on_close.take());

        let header = Container::new(
            crate::row![Text::new(self.title), space::horizontal(), close]
                .align_y(crate::core::alignment::Vertical::Center),
        )
        .padding(10)
        .width(Length::Fill)
        .style(move |theme| self.status.style(theme));

        Container::new(
            crate::column![header, rule::horizontal(1), Text::new(self.body),]
                .spacing(10)
                .padding(10),
        )
        .style(container::rounded_box)
        .width(self.width)
        .padding(self.padding)
        ._boxed()
    }
}
