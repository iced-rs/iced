pub mod cache;
pub mod paragraph;

pub use cache::Cache;
pub use paragraph::Paragraph;

pub use cosmic_text;

use crate::core::font::{self, Font};
use crate::core::text::Shaping;
use crate::core::Size;

use std::borrow::Cow;
use std::sync::{self, Arc, RwLock};

#[allow(missing_debug_implementations)]
pub struct FontSystem {
    raw: RwLock<cosmic_text::FontSystem>,
    version: Version,
}

impl FontSystem {
    pub fn new() -> Self {
        FontSystem {
            raw: RwLock::new(cosmic_text::FontSystem::new_with_fonts(
                [cosmic_text::fontdb::Source::Binary(Arc::new(
                    include_bytes!("../fonts/Iced-Icons.ttf").as_slice(),
                ))]
                .into_iter(),
            )),
            version: Version::default(),
        }
    }

    pub fn get_mut(&mut self) -> &mut cosmic_text::FontSystem {
        self.raw.get_mut().expect("Lock font system")
    }

    pub fn write(
        &self,
    ) -> (sync::RwLockWriteGuard<'_, cosmic_text::FontSystem>, Version) {
        (self.raw.write().expect("Write font system"), self.version)
    }

    pub fn load_font(&mut self, bytes: Cow<'static, [u8]>) {
        let _ = self.get_mut().db_mut().load_font_source(
            cosmic_text::fontdb::Source::Binary(Arc::new(bytes.into_owned())),
        );

        self.version = Version(self.version.0 + 1);
    }

    pub fn version(&self) -> Version {
        self.version
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Version(u32);

impl Default for FontSystem {
    fn default() -> Self {
        Self::new()
    }
}

pub fn measure(buffer: &cosmic_text::Buffer) -> Size {
    let (width, total_lines) = buffer
        .layout_runs()
        .fold((0.0, 0usize), |(width, total_lines), run| {
            (run.line_w.max(width), total_lines + 1)
        });

    Size::new(width, total_lines as f32 * buffer.metrics().line_height)
}

pub fn to_attributes(font: Font) -> cosmic_text::Attrs<'static> {
    cosmic_text::Attrs::new()
        .family(to_family(font.family))
        .weight(to_weight(font.weight))
        .stretch(to_stretch(font.stretch))
        .style(to_style(font.style))
}

fn to_family(family: font::Family) -> cosmic_text::Family<'static> {
    match family {
        font::Family::Name(name) => cosmic_text::Family::Name(name),
        font::Family::SansSerif => cosmic_text::Family::SansSerif,
        font::Family::Serif => cosmic_text::Family::Serif,
        font::Family::Cursive => cosmic_text::Family::Cursive,
        font::Family::Fantasy => cosmic_text::Family::Fantasy,
        font::Family::Monospace => cosmic_text::Family::Monospace,
    }
}

fn to_weight(weight: font::Weight) -> cosmic_text::Weight {
    match weight {
        font::Weight::Thin => cosmic_text::Weight::THIN,
        font::Weight::ExtraLight => cosmic_text::Weight::EXTRA_LIGHT,
        font::Weight::Light => cosmic_text::Weight::LIGHT,
        font::Weight::Normal => cosmic_text::Weight::NORMAL,
        font::Weight::Medium => cosmic_text::Weight::MEDIUM,
        font::Weight::Semibold => cosmic_text::Weight::SEMIBOLD,
        font::Weight::Bold => cosmic_text::Weight::BOLD,
        font::Weight::ExtraBold => cosmic_text::Weight::EXTRA_BOLD,
        font::Weight::Black => cosmic_text::Weight::BLACK,
    }
}

fn to_stretch(stretch: font::Stretch) -> cosmic_text::Stretch {
    match stretch {
        font::Stretch::UltraCondensed => cosmic_text::Stretch::UltraCondensed,
        font::Stretch::ExtraCondensed => cosmic_text::Stretch::ExtraCondensed,
        font::Stretch::Condensed => cosmic_text::Stretch::Condensed,
        font::Stretch::SemiCondensed => cosmic_text::Stretch::SemiCondensed,
        font::Stretch::Normal => cosmic_text::Stretch::Normal,
        font::Stretch::SemiExpanded => cosmic_text::Stretch::SemiExpanded,
        font::Stretch::Expanded => cosmic_text::Stretch::Expanded,
        font::Stretch::ExtraExpanded => cosmic_text::Stretch::ExtraExpanded,
        font::Stretch::UltraExpanded => cosmic_text::Stretch::UltraExpanded,
    }
}

fn to_style(style: font::Style) -> cosmic_text::Style {
    match style {
        font::Style::Normal => cosmic_text::Style::Normal,
        font::Style::Italic => cosmic_text::Style::Italic,
        font::Style::Oblique => cosmic_text::Style::Oblique,
    }
}

pub fn to_shaping(shaping: Shaping) -> cosmic_text::Shaping {
    match shaping {
        Shaping::Basic => cosmic_text::Shaping::Basic,
        Shaping::Advanced => cosmic_text::Shaping::Advanced,
    }
}
