//! Use default styling attributes to inherit styles.
use iced_native::Color;

/// Some default styling attributes.
#[derive(Debug, Clone, Copy)]
pub struct Defaults {
    /// Text styling
    pub text: Text,
}

impl Default for Defaults {
    fn default() -> Defaults {
        Defaults {
            text: Text::default(),
        }
    }
}

/// Some default text styling attributes.
#[derive(Debug, Clone, Copy)]
pub struct Text {
    /// The default color of text
    pub color: Color,
}

impl Default for Text {
    fn default() -> Text {
        Text {
            color: Color::BLACK,
        }
    }
}
