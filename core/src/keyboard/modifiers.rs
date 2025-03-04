use bitflags::bitflags;

bitflags! {
    /// The current state of the keyboard modifiers.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Modifiers: u32{
        /// The "shift" key.
        const SHIFT = 0b100;
        // const LSHIFT = 0b010 << 0;
        // const RSHIFT = 0b001 << 0;
        //
        /// The "control" key.
        const CTRL = 0b100 << 3;
        // const LCTRL = 0b010 << 3;
        // const RCTRL = 0b001 << 3;
        //
        /// The "alt" key.
        const ALT = 0b100 << 6;
        // const LALT = 0b010 << 6;
        // const RALT = 0b001 << 6;
        //
        /// The "windows" key on Windows, "command" key on Mac, and
        /// "super" key on Linux.
        const LOGO = 0b100 << 9;
        // const LLOGO = 0b010 << 9;
        // const RLOGO = 0b001 << 9;
    }
}

impl Modifiers {
    /// The "command" key.
    ///
    /// This is normally the main modifier to be used for hotkeys.
    ///
    /// On macOS, this is equivalent to `Self::LOGO`.
    /// Otherwise, this is equivalent to `Self::CTRL`.
    pub const COMMAND: Self = if cfg!(target_os = "macos") {
        Self::LOGO
    } else {
        Self::CTRL
    };

    /// Returns true if the [`SHIFT`] key is pressed in the [`Modifiers`].
    ///
    /// [`SHIFT`]: Self::SHIFT
    pub fn shift(self) -> bool {
        self.contains(Self::SHIFT)
    }

    /// Returns true if the [`CTRL`] key is pressed in the [`Modifiers`].
    ///
    /// [`CTRL`]: Self::CTRL
    pub fn control(self) -> bool {
        self.contains(Self::CTRL)
    }

    /// Returns true if the [`ALT`] key is pressed in the [`Modifiers`].
    ///
    /// [`ALT`]: Self::ALT
    pub fn alt(self) -> bool {
        self.contains(Self::ALT)
    }

    /// Returns true if the [`LOGO`] key is pressed in the [`Modifiers`].
    ///
    /// [`LOGO`]: Self::LOGO
    pub fn logo(self) -> bool {
        self.contains(Self::LOGO)
    }

    /// Returns true if a "command key" is pressed in the [`Modifiers`].
    ///
    /// The "command key" is the main modifier key used to issue commands in the
    /// current platform. Specifically:
    ///
    /// - It is the `logo` or command key (⌘) on macOS
    /// - It is the `control` key on other platforms
    pub fn command(self) -> bool {
        #[cfg(target_os = "macos")]
        let is_pressed = self.logo();

        #[cfg(not(target_os = "macos"))]
        let is_pressed = self.control();

        is_pressed
    }

    /// Returns true if the "jump key" is pressed in the [`Modifiers`].
    ///
    /// The "jump key" is the modifier key used to widen text motions. It is the `Alt`
    /// key in macOS and the `Ctrl` key in other platforms.
    pub fn jump(self) -> bool {
        if cfg!(target_os = "macos") {
            self.alt()
        } else {
            self.control()
        }
    }

    /// Returns true if the "command key" is pressed on a macOS device.
    ///
    /// This is relevant for macOS-specific actions (e.g. `⌘ + ArrowLeft` moves the cursor
    /// to the beginning of the line).
    pub fn macos_command(self) -> bool {
        if cfg!(target_os = "macos") {
            self.logo()
        } else {
            false
        }
    }
}
