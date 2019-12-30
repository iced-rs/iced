use iced_native::Color;

#[derive(Debug, Clone, Copy)]
pub struct Defaults {
    pub text: Text,
}

impl Default for Defaults {
    fn default() -> Defaults {
        Defaults {
            text: Text::default(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Text {
    pub color: Color,
}

impl Default for Text {
    fn default() -> Text {
        Text {
            color: Color::BLACK,
        }
    }
}
