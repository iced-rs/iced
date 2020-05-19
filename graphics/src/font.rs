#[cfg(feature = "font-source")]
mod source;

#[cfg(feature = "font-source")]
pub use source::Source;

#[cfg(feature = "font-source")]
pub use font_kit::{
    error::SelectionError as LoadError, family_name::FamilyName as Family,
};
