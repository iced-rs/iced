//! Draw text.
pub mod cache;
pub mod editor;
pub mod paragraph;

pub use cache::Cache;
pub use editor::Editor;
pub use paragraph::Paragraph;

pub use cosmic_text;

use crate::core::alignment;
use crate::core::font::{self, Font};
use crate::core::text::Shaping;
use crate::core::{Color, Pixels, Point, Rectangle, Size, Transformation};

use once_cell::sync::OnceCell;
use std::borrow::Cow;
use std::sync::{Arc, RwLock, Weak};

/// A text primitive.
#[derive(Debug, Clone, PartialEq)]
pub enum Text {
    /// A paragraph.
    #[allow(missing_docs)]
    Paragraph {
        paragraph: paragraph::Weak,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
        transformation: Transformation,
    },
    /// An editor.
    #[allow(missing_docs)]
    Editor {
        editor: editor::Weak,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
        transformation: Transformation,
    },
    /// Some cached text.
    Cached {
        /// The contents of the text.
        content: String,
        /// The bounds of the text.
        bounds: Rectangle,
        /// The color of the text.
        color: Color,
        /// The size of the text in logical pixels.
        size: Pixels,
        /// The line height of the text.
        line_height: Pixels,
        /// The font of the text.
        font: Font,
        /// The horizontal alignment of the text.
        horizontal_alignment: alignment::Horizontal,
        /// The vertical alignment of the text.
        vertical_alignment: alignment::Vertical,
        /// The shaping strategy of the text.
        shaping: Shaping,
        /// The clip bounds of the text.
        clip_bounds: Rectangle,
    },
    /// Some raw text.
    #[allow(missing_docs)]
    Raw {
        raw: Raw,
        transformation: Transformation,
    },
}

impl Text {
    /// Returns the visible bounds of the [`Text`].
    pub fn visible_bounds(&self) -> Option<Rectangle> {
        let (bounds, horizontal_alignment, vertical_alignment) = match self {
            Text::Paragraph {
                position,
                paragraph,
                clip_bounds,
                transformation,
                ..
            } => (
                Rectangle::new(*position, paragraph.min_bounds)
                    .intersection(clip_bounds)
                    .map(|bounds| bounds * *transformation),
                Some(paragraph.horizontal_alignment),
                Some(paragraph.vertical_alignment),
            ),
            Text::Editor {
                editor,
                position,
                clip_bounds,
                transformation,
                ..
            } => (
                Rectangle::new(*position, editor.bounds)
                    .intersection(clip_bounds)
                    .map(|bounds| bounds * *transformation),
                None,
                None,
            ),
            Text::Cached {
                bounds,
                clip_bounds,
                horizontal_alignment,
                vertical_alignment,
                ..
            } => (
                bounds.intersection(clip_bounds),
                Some(*horizontal_alignment),
                Some(*vertical_alignment),
            ),
            Text::Raw { raw, .. } => (Some(raw.clip_bounds), None, None),
        };

        let mut bounds = bounds?;

        if let Some(alignment) = horizontal_alignment {
            match alignment {
                alignment::Horizontal::Left => {}
                alignment::Horizontal::Center => {
                    bounds.x -= bounds.width / 2.0;
                }
                alignment::Horizontal::Right => {
                    bounds.x -= bounds.width;
                }
            }
        }

        if let Some(alignment) = vertical_alignment {
            match alignment {
                alignment::Vertical::Top => {}
                alignment::Vertical::Center => {
                    bounds.y -= bounds.height / 2.0;
                }
                alignment::Vertical::Bottom => {
                    bounds.y -= bounds.height;
                }
            }
        }

        Some(bounds)
    }
}

/// The regular variant of the [Fira Sans] font.
///
/// It is loaded as part of the default fonts in Wasm builds.
///
/// [Fira Sans]: https://mozilla.github.io/Fira/
#[cfg(all(target_arch = "wasm32", feature = "fira-sans"))]
pub const FIRA_SANS_REGULAR: &'static [u8] =
    include_bytes!("../fonts/FiraSans-Regular.ttf").as_slice();

/// Returns the global [`FontSystem`].
pub fn font_system() -> &'static RwLock<FontSystem> {
    static FONT_SYSTEM: OnceCell<RwLock<FontSystem>> = OnceCell::new();

    FONT_SYSTEM.get_or_init(|| {
        RwLock::new(FontSystem {
            raw: cosmic_text::FontSystem::new_with_fonts([
                cosmic_text::fontdb::Source::Binary(Arc::new(
                    include_bytes!("../fonts/Iced-Icons.ttf").as_slice(),
                )),
                #[cfg(all(target_arch = "wasm32", feature = "fira-sans"))]
                cosmic_text::fontdb::Source::Binary(Arc::new(
                    include_bytes!("../fonts/FiraSans-Regular.ttf").as_slice(),
                )),
            ]),
            version: Version::default(),
        })
    })
}

/// A set of system fonts.
#[allow(missing_debug_implementations)]
pub struct FontSystem {
    raw: cosmic_text::FontSystem,
    version: Version,
}

impl FontSystem {
    /// Returns the raw [`cosmic_text::FontSystem`].
    pub fn raw(&mut self) -> &mut cosmic_text::FontSystem {
        &mut self.raw
    }

    /// Loads a font from its bytes.
    pub fn load_font(&mut self, bytes: Cow<'static, [u8]>) {
        let _ = self.raw.db_mut().load_font_source(
            cosmic_text::fontdb::Source::Binary(Arc::new(bytes.into_owned())),
        );

        self.version = Version(self.version.0 + 1);
    }

    /// Returns the current [`Version`] of the [`FontSystem`].
    ///
    /// Loading a font will increase the version of a [`FontSystem`].
    pub fn version(&self) -> Version {
        self.version
    }
}

/// A version number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Version(u32);

/// A weak reference to a [`cosmic-text::Buffer`] that can be drawn.
#[derive(Debug, Clone)]
pub struct Raw {
    /// A weak reference to a [`cosmic_text::Buffer`].
    pub buffer: Weak<cosmic_text::Buffer>,
    /// The position of the text.
    pub position: Point,
    /// The color of the text.
    pub color: Color,
    /// The clip bounds of the text.
    pub clip_bounds: Rectangle,
}

impl PartialEq for Raw {
    fn eq(&self, _other: &Self) -> bool {
        // TODO: There is no proper way to compare raw buffers
        // For now, no two instances of `Raw` text will be equal.
        // This should be fine, but could trigger unnecessary redraws
        // in the future.
        false
    }
}

/// Measures the dimensions of the given [`cosmic_text::Buffer`].
pub fn measure(buffer: &cosmic_text::Buffer) -> Size {
    let (width, total_lines) = buffer
        .layout_runs()
        .fold((0.0, 0usize), |(width, total_lines), run| {
            (run.line_w.max(width), total_lines + 1)
        });

    Size::new(width, total_lines as f32 * buffer.metrics().line_height)
}

/// Returns the attributes of the given [`Font`].
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

/// Converts some [`Shaping`] strategy to a [`cosmic_text::Shaping`] strategy.
pub fn to_shaping(shaping: Shaping) -> cosmic_text::Shaping {
    match shaping {
        Shaping::Basic => cosmic_text::Shaping::Basic,
        Shaping::Advanced => cosmic_text::Shaping::Advanced,
    }
}

/// Converts some [`Color`] to a [`cosmic_text::Color`].
pub fn to_color(color: Color) -> cosmic_text::Color {
    let [r, g, b, a] = color.into_rgba8();

    cosmic_text::Color::rgba(r, g, b, a)
}
