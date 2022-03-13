mod dark;
mod test;
mod ugly;

use iced::{Color, Styling};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme1 {
    /// The default iced theme
    IcedDefault,
    /// The dark theme
    Dark,
    /// The ugliest color theme to exist
    Ugly,
}

impl Styling {
    pub const ALL: [Styling; 3] = [Styling::IcedDefault, Styling::Dark, Styling::Ugly];
}

impl std::default::Default for Styling {
    fn default() -> Styling {
        Styling::IcedDefault
    }
}

impl From<Styling> for Styling {
    fn from(theme: Styling) -> Self {
        match theme {
            Styling::IcedDefault => Styling::default(),
            Styling::Dark => dark::get_style(),
            Styling::Ugly => ugly::get_style(),
        }
    }
}
