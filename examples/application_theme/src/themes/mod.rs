mod dark;
mod ugly;

use iced::Style;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    /// The default iced theme
    IcedDefault,
    /// The dark theme
    Dark,
    /// The ugliest color theme to exist
    Ugly,
}

impl Theme {
    pub const ALL: [Theme; 3] = [Theme::IcedDefault, Theme::Dark, Theme::Ugly];
}

impl std::default::Default for Theme {
    fn default() -> Theme {
        Theme::IcedDefault
    }
}

impl From<Theme> for Style {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::IcedDefault => Style::default(),
            Theme::Dark => dark::get_style(),
            Theme::Ugly => ugly::get_style(),
        }
    }
}
