#[cfg(feature = "font-source")]
mod source;

#[cfg(feature = "font-source")]
pub use source::Source;

#[cfg(feature = "font-source")]
pub use font_kit::{
    error::SelectionError as LoadError, family_name::FamilyName as Family,
};

#[cfg(feature = "font-fallback")]
pub const FALLBACK: &[u8] = include_bytes!("../fonts/Lato-Regular.ttf");

#[cfg(feature = "font-icons")]
pub const ICONS: iced_native::Font = iced_native::Font::External {
    name: "iced_wgpu icons",
    bytes: include_bytes!("../fonts/Icons.ttf"),
};

#[cfg(feature = "font-icons")]
pub const CHECKMARK_ICON: char = '\u{F00C}';
