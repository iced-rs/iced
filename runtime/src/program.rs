//! Build interactive programs using The Elm Architecture.
use crate::Command;

use iced_core::text;
use iced_core::{window::Id, Element, Renderer};

mod state;

pub use state::State;

/// The core of a user interface application following The Elm Architecture.
pub trait Program: Sized {
    /// The graphics backend to use to draw the [`Program`].
    type Renderer: Renderer + text::Renderer;

    /// The type of __messages__ your [`Program`] will produce.
    type Message: std::fmt::Debug + Send;

    /// Handles a __message__ and updates the state of the [`Program`].
    ///
    /// This is where you define your __update logic__. All the __messages__,
    /// produced by either user interactions or commands, will be handled by
    /// this method.
    ///
    /// Any [`Command`] returned will be executed immediately in the
    /// background by shells.
    fn update(&mut self, message: Self::Message) -> Command<Self::Message>;

    /// Returns the widgets to display in the [`Program`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    fn view(&self, id: Id) -> Element<'_, Self::Message, Self::Renderer>;
}
