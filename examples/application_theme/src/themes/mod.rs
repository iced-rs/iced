mod dark;
mod test;
mod ugly;

use iced::{Color, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme1 {
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

impl From<Theme> for Theme {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::IcedDefault => Theme::default(),
            Theme::Dark => dark::get_style(),
            Theme::Ugly => ugly::get_style(),
        }
    }
}
