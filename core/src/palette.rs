use crate::Color;

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

impl Default for Palette {
    fn default() -> Self {
        let std_pal = std::env::vars()
            .find_map(|(key, value)| {
                if key == "ICED_PALETTE" {
                    Some(value)
                } else {
                    None
                }
            })
            .unwrap_or(String::new());

        match std_pal.as_str() {
            "dark" => Self {
                window: Color::from_rgb8(49, 54, 59),
                window_text: Color::from_rgb8(239, 240, 241),
                base: Color::from_rgb8(35, 38, 41),
                alternate_base: Color::from_rgb8(49, 54, 59),
                tooltip_base: Color::from_rgb8(49, 54, 59),
                tooltip_text: Color::from_rgb8(239, 240, 241),
                placeholder_text: Color::from_rgb8(189, 195, 199),
                text: Color::from_rgb8(239, 240, 241),
                button: Color::from_rgb8(49, 54, 59),
                button_text: Color::from_rgb8(239, 240, 241),
                highlight: Color::from_rgb8(61, 174, 233),
                highlight_text: Color::from_rgb8(61, 174, 233),
                link: Color::from_rgb8(41, 128, 185),
                link_visited: Color::from_rgb8(127, 140, 141),
                positive: Color::from_rgb8(39, 174, 96),
                neutral: Color::from_rgb8(246, 116, 0),
                negative: Color::from_rgb8(218, 68, 83),
            },
            "light" => Self {
                window: Color::from_rgb8(239, 240, 241),
                window_text: Color::from_rgb8(35, 38, 39),
                base: Color::from_rgb8(252, 252, 252),
                alternate_base: Color::from_rgb8(248, 247, 246),
                tooltip_base: Color::from_rgb8(252, 252, 252),
                tooltip_text: Color::from_rgb8(35, 38, 39),
                placeholder_text: Color::from_rgb8(136, 135, 134),
                text: Color::from_rgb8(35, 38, 39),
                button: Color::from_rgb8(239, 240, 241),
                button_text: Color::from_rgb8(35, 38, 39),
                highlight: Color::from_rgb8(61, 174, 233),
                highlight_text: Color::from_rgb8(252, 252, 252),
                link: Color::from_rgb8(0, 87, 174),
                link_visited: Color::from_rgb8(69, 40, 134),
                positive: Color::from_rgb8(0, 110, 40),
                neutral: Color::from_rgb8(176, 128, 0),
                negative: Color::from_rgb8(191, 3, 3),
            },
            _ => Self {
                window: Color::from_rgb8(255, 255, 255),
                window_text: Color::from_rgb8(0, 0, 0),
                base: Color::from_rgb8(255, 255, 255),
                alternate_base: Color::from_rgb8(240, 240, 240),
                tooltip_base: Color::from_rgb8(255, 255, 255),
                tooltip_text: Color::from_rgb8(0, 0, 0),
                placeholder_text: Color::from_rgb8(178, 178, 178),
                text: Color::from_rgb8(0, 0, 0),
                button: Color::from_rgb8(255, 255, 255),
                button_text: Color::from_rgb8(0, 0, 0),
                highlight: Color::from_rgb8(110, 145, 180),
                highlight_text: Color::from_rgb8(255, 255, 255),
                link: Color::from_rgb8(0, 0, 255),
                link_visited: Color::from_rgb8(69, 40, 134),
                positive: Color::from_rgb8(0, 255, 0),
                neutral: Color::from_rgb8(255, 255, 0),
                negative: Color::from_rgb8(255, 0, 0),
            },
        }
    }
}
