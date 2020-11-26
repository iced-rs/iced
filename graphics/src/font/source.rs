use crate::font::{Family, LoadError};

/// A font source that can find and load system fonts.
#[allow(missing_debug_implementations)]
pub struct Source {
    raw: font_kit::source::SystemSource,
}

impl Source {
    /// Creates a new [`Source`].
    pub fn new() -> Self {
        Source {
            raw: font_kit::source::SystemSource::new(),
        }
    }

    /// Finds and loads a font matching the set of provided family priorities.
    pub fn load(&self, families: &[Family]) -> Result<Vec<u8>, LoadError> {
        let font = self.raw.select_best_match(
            families,
            &font_kit::properties::Properties::default(),
        )?;

        match font {
            font_kit::handle::Handle::Path { path, .. } => {
                use std::io::Read;

                let mut buf = Vec::new();
                let mut reader = std::fs::File::open(path).expect("Read font");
                let _ = reader.read_to_end(&mut buf);

                Ok(buf)
            }
            font_kit::handle::Handle::Memory { bytes, .. } => {
                Ok(bytes.as_ref().clone())
            }
        }
    }
}
