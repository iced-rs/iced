/// The window settings of an application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    /// The position of the window.
    pub position: Option<(u32, u32)>,

    /// The size of the window.
    pub size: (u32, u32),

    /// Whether the window should be resizable or not.
    pub resizable: bool,

    /// Whether the window should have a border, a title bar, etc. or not.
    pub decorations: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            position: None,
            size: (1024, 768),
            resizable: true,
            decorations: true,
        }
    }
}
