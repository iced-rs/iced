use crate::Color;

const LIGHT_PALETTE: Palette = Palette {
    window: Color::from_rgb(0.937, 0.941, 0.945),
    window_text: Color::from_rgb(0.137, 0.149, 0.153),
    base: Color::from_rgb(0.988, 0.988, 0.988),
    alternate_base: Color::from_rgb(0.973, 0.969, 0.965),
    tooltip_base: Color::from_rgb(0.988, 0.988, 0.988),
    tooltip_text: Color::from_rgb(0.137, 0.149, 0.153),
    placeholder_text: Color::from_rgb(0.533, 0.529, 0.525),
    text: Color::from_rgb(0.137, 0.149, 0.153),
    button: Color::from_rgb(0.937, 0.941, 0.945),
    button_text: Color::from_rgb(0.137, 0.149, 0.153),
    highlight: Color::from_rgb(0.239, 0.682, 0.914),
    highlight_text: Color::from_rgb(0.988, 0.988, 0.988),
    link: Color::from_rgb(0.0, 0.341, 0.682),
    link_visited: Color::from_rgb(0.271, 0.157, 0.525),
    positive: Color::from_rgb(0.0, 0.431, 0.157),
    neutral: Color::from_rgb(0.69, 0.502, 0.0),
    negative: Color::from_rgb(0.749, 0.012, 0.012),
};

const DARK_PALETTE: Palette = Palette {
    window: Color::from_rgb(0.192, 0.212, 0.231),
    window_text: Color::from_rgb(0.937, 0.941, 0.945),
    base: Color::from_rgb(0.137, 0.149, 0.161),
    alternate_base: Color::from_rgb(0.192, 0.212, 0.231),
    tooltip_base: Color::from_rgb(0.192, 0.212, 0.231),
    tooltip_text: Color::from_rgb(0.937, 0.941, 0.945),
    placeholder_text: Color::from_rgb(0.741, 0.765, 0.780),
    text: Color::from_rgb(0.937, 0.941, 0.945),
    button: Color::from_rgb(0.192, 0.212, 0.231),
    button_text: Color::from_rgb(0.937, 0.941, 0.945),
    highlight: Color::from_rgb(0.239, 0.682, 0.914),
    highlight_text: Color::from_rgb(0.239, 0.682, 0.914),
    link: Color::from_rgb(0.161, 0.502, 0.725),
    link_visited: Color::from_rgb(0.5, 0.549, 0.553),
    positive: Color::from_rgb(0.153, 0.682, 0.376),
    neutral: Color::from_rgb(0.965, 0.455, 0.0),
    negative: Color::from_rgb(0.855, 0.267, 0.325),
};

const FALLBACK_PALETTE: Palette = Palette {
    window: Color::from_rgb(1.0, 1.0, 1.0),
    window_text: Color::from_rgb(0.0, 0.0, 0.0),
    base: Color::from_rgb(1.0, 1.0, 1.0),
    alternate_base: Color::from_rgb(0.941, 0.941, 0.941),
    tooltip_base: Color::from_rgb(1.0, 1.0, 1.0),
    tooltip_text: Color::from_rgb(0.0, 0.0, 0.0),
    placeholder_text: Color::from_rgb(0.698, 0.698, 0.698),
    text: Color::from_rgb(0.0, 0.0, 0.0),
    button: Color::from_rgb(1.0, 1.0, 1.0),
    button_text: Color::from_rgb(0.0, 0.0, 0.0),
    highlight: Color::from_rgb(0.431, 0.569, 0.706),
    highlight_text: Color::from_rgb(1.0, 1.0, 1.0),
    link: Color::from_rgb(0.0, 0.0, 1.0),
    link_visited: Color::from_rgb(0.271, 0.157, 0.525),
    positive: Color::from_rgb(0.0, 1.0, 0.0),
    neutral: Color::from_rgb(1.0, 1.0, 0.0),
    negative: Color::from_rgb(1.0, 0.0, 0.0),
};

/// Struct that holds values for a range of different colors.
/// It can be used to create custom widgets that look like native ones.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    /// The color for the window background
    pub window: Color,

    /// The primary foreground color
    pub window_text: Color,

    /// The color for text entry widgets and drop down menus
    pub base: Color,

    /// An alternate base color for alternating lists
    pub alternate_base: Color,

    /// The color for tooltip backgrounds
    pub tooltip_base: Color,

    /// The color for test in tooltips
    pub tooltip_text: Color,

    /// The color for placeholder text in text input widgets and the like
    pub placeholder_text: Color,

    /// The primary text color
    pub text: Color,

    /// The button background color
    pub button: Color,

    /// The color for button text
    pub button_text: Color,

    /// The primary highlight color
    pub highlight: Color,

    /// The color for highlighted text
    pub highlight_text: Color,

    /// The color for links
    pub link: Color,

    /// The color for already visited links
    pub link_visited: Color,

    /// The color to give something positive. It is usually some sort of green.
    pub positive: Color,

    /// The color to give something neutral. It is usually some sort of yellow.
    pub neutral: Color,

    /// The color to give something negative. It is usually some sort of red.
    pub negative: Color,
}

impl Palette {
    /// Returns a reference to the default palette.
    pub fn default() -> &'static Self {
        let std_pal = std::env::vars()
            .find_map(|(key, value)| {
                if key == "ICED_PALETTE" {
                    Some(value)
                } else {
                    None
                }
            })
            .unwrap_or_else(String::new);

        match std_pal.as_str() {
            "dark" => &DARK_PALETTE,
            "light" => &LIGHT_PALETTE,
            _ => &FALLBACK_PALETTE,
        }
    }
}
