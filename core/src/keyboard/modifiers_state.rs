/// The current state of the keyboard modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ModifiersState {
    /// Whether a shift key is pressed
    pub shift: bool,

    /// Whether a control key is pressed
    pub control: bool,

    /// Whether an alt key is pressed
    pub alt: bool,

    /// Whether a logo key is pressed (e.g. windows key, command key...)
    pub logo: bool,
}

impl ModifiersState {
    /// Returns true if the current [`ModifiersState`] has a "command key"
    /// pressed.
    ///
    /// The "command key" is the main modifier key used to issue commands in the
    /// current platform. Specifically:
    ///
    /// - It is the `logo` or command key (⌘) on macOS
    /// - It is the `control` key on other platforms
    ///
    /// [`ModifiersState`]: struct.ModifiersState.html
    pub fn is_command_pressed(self) -> bool {
        #[cfg(target_os = "macos")]
        let is_pressed = self.logo;

        #[cfg(not(target_os = "macos"))]
        let is_pressed = self.control;

        is_pressed
    }

    /// Returns true if the current [`ModifiersState`] has at least the same
    /// modifiers enabled as the given value, and false otherwise.
    ///
    /// [`ModifiersState`]: struct.ModifiersState.html
    pub fn matches(&self, modifiers: ModifiersState) -> bool {
        let shift = !modifiers.shift || self.shift;
        let control = !modifiers.control || self.control;
        let alt = !modifiers.alt || self.alt;
        let logo = !modifiers.logo || self.logo;

        shift && control && alt && logo
    }
}
