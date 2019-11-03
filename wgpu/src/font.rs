pub use font_kit::family_name::FamilyName as Family;

pub struct Source {
    raw: font_kit::source::SystemSource,
}

impl Source {
    pub fn new() -> Self {
        Source {
            raw: font_kit::source::SystemSource::new(),
        }
    }

    pub fn load(&self, families: &[Family]) -> Vec<u8> {
        let font = self
            .raw
            .select_best_match(
                families,
                &font_kit::properties::Properties::default(),
            )
            .expect("Find font");

        match font {
            font_kit::handle::Handle::Path { path, .. } => {
                use std::io::Read;

                let mut buf = Vec::new();
                let mut reader = std::fs::File::open(path).expect("Read font");
                let _ = reader.read_to_end(&mut buf);

                buf
            }
            font_kit::handle::Handle::Memory { bytes, .. } => {
                bytes.as_ref().clone()
            }
        }
    }
}
