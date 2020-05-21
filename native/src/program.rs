//! Build interactive programs using The Elm Architecture.
use crate::{Command, Element, Renderer};

mod state;

pub use state::State;

/// An interactive, native cross-platform program.
pub trait Program: Sized {
    /// The graphics backend to use to draw the [`Program`].
    ///
    /// [`Program`]: trait.Program.html
    type Renderer: Renderer;

    /// The type of __messages__ your [`Program`] will produce.
    ///
    /// [`Application`]: trait.Program.html
    type Message: std::fmt::Debug + Send;

    /// Handles a __message__ and updates the state of the [`Program`].
    ///
    /// This is where you define your __update logic__. All the __messages__,
    /// produced by either user interactions or commands, will be handled by
    /// this method.
    ///
    /// Any [`Command`] returned will be executed immediately in the
    /// background by shells.
    ///
    /// [`Program`]: trait.Application.html
    /// [`Command`]: struct.Command.html
    fn update(&mut self, message: Self::Message) -> Command<Self::Message>;

    /// Returns the widgets to display in the [`Program`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    ///
    /// [`Program`]: trait.Application.html
    fn view(&mut self) -> Element<'_, Self::Message, Self::Renderer>;
}
