//! Load and use fonts.
use std::hash::Hash;

/// A font.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Font {
    /// The [`Family`] of the [`Font`].
    pub family: Family,
    /// The [`Weight`] of the [`Font`].
    pub weight: Weight,
    /// The [`Stretch`] of the [`Font`].
    pub stretch: Stretch,
    /// The [`Style`] of the [`Font`].
    pub style: Style,
}

impl Font {
    /// A non-monospaced sans-serif font with normal [`Weight`].
    pub const DEFAULT: Font = Font {
        family: Family::SansSerif,
        weight: Weight::Normal,
        stretch: Stretch::Normal,
        style: Style::Normal,
    };

    /// A monospaced font with normal [`Weight`].
    pub const MONOSPACE: Font = Font {
        family: Family::Monospace,
        ..Self::DEFAULT
    };

    /// Creates a [`Font`] with the given [`Family::Name`] and default attributes.
    pub const fn new(name: &'static str) -> Self {
        Self {
            family: Family::Name(name),
            ..Self::DEFAULT
        }
    }

    /// Creates a [`Font`] with the given [`Family`] and default attributes.
    pub fn with_family(family: impl Into<Family>) -> Self {
        Font {
            family: family.into(),
            ..Self::DEFAULT
        }
    }

    /// Sets the [`Weight`] of the [`Font`].
    pub const fn weight(self, weight: Weight) -> Self {
        Self { weight, ..self }
    }

    /// Sets the [`Stretch`] of the [`Font`].
    pub const fn stretch(self, stretch: Stretch) -> Self {
        Self { stretch, ..self }
    }

    /// Sets the [`Style`] of the [`Font`].
    pub const fn style(self, style: Style) -> Self {
        Self { style, ..self }
    }
}

/// Returns the cosmic_text attributes of the given [`Font`].
impl From<Font> for cosmic_text::Attrs<'static> {
    fn from(font: Font) -> cosmic_text::Attrs<'static> {
        cosmic_text::Attrs::new()
            .family(font.family.into())
            .weight(font.weight.into())
            .stretch(font.stretch.into())
            .style(font.style.into())
    }
}

impl From<&'static str> for Font {
    fn from(name: &'static str) -> Self {
        Font::new(name)
    }
}

impl From<Family> for Font {
    fn from(family: Family) -> Self {
        Font::with_family(family)
    }
}

/// A font family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Family {
    /// The name of a font family of choice.
    Name(&'static str),

    /// Serif fonts represent the formal text style for a script.
    Serif,

    /// Glyphs in sans-serif fonts, as the term is used in CSS, are generally low
    /// contrast and have stroke endings that are plain — without any flaring,
    /// cross stroke, or other ornamentation.
    #[default]
    SansSerif,

    /// Glyphs in cursive fonts generally use a more informal script style, and
    /// the result looks more like handwritten pen or brush writing than printed
    /// letterwork.
    Cursive,

    /// Fantasy fonts are primarily decorative or expressive fonts that contain
    /// decorative or expressive representations of characters.
    Fantasy,

    /// The sole criterion of a monospace font is that all glyphs have the same
    /// fixed width.
    Monospace,
}

impl Family {
    /// A list of all the different standalone family variants.
    pub const VARIANTS: &[Self] = &[
        Self::Serif,
        Self::SansSerif,
        Self::Cursive,
        Self::Fantasy,
        Self::Monospace,
    ];

    /// Creates a [`Family::Name`] from the given string.
    ///
    /// The name is interned in a global cache and never freed.
    pub fn name(name: &str) -> Self {
        use rustc_hash::FxHashSet;
        use std::sync::{LazyLock, Mutex};

        static NAMES: LazyLock<Mutex<FxHashSet<&'static str>>> = LazyLock::new(Mutex::default);

        let mut names = NAMES.lock().expect("lock font name cache");

        let Some(name) = names.get(name) else {
            let name: &'static str = name.to_owned().leak();
            let _ = names.insert(name);

            return Self::Name(name);
        };

        Self::Name(name)
    }
}

impl From<cosmic_text::Family<'static>> for Family {
    fn from(family: cosmic_text::Family<'static>) -> Family {
        match family {
            cosmic_text::Family::Name(name) => Family::Name(name),
            cosmic_text::Family::SansSerif => Family::SansSerif,
            cosmic_text::Family::Serif => Family::Serif,
            cosmic_text::Family::Cursive => Family::Cursive,
            cosmic_text::Family::Fantasy => Family::Fantasy,
            cosmic_text::Family::Monospace => Family::Monospace,
        }
    }
}

impl From<Family> for cosmic_text::Family<'static> {
    fn from(family: Family) -> cosmic_text::Family<'static> {
        match family {
            Family::Name(name) => cosmic_text::Family::Name(name),
            Family::SansSerif => cosmic_text::Family::SansSerif,
            Family::Serif => cosmic_text::Family::Serif,
            Family::Cursive => cosmic_text::Family::Cursive,
            Family::Fantasy => cosmic_text::Family::Fantasy,
            Family::Monospace => cosmic_text::Family::Monospace,
        }
    }
}

impl From<&str> for Family {
    fn from(name: &str) -> Self {
        Family::name(name)
    }
}

impl std::fmt::Display for Family {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Family::Name(name) => name,
            Family::Serif => "Serif",
            Family::SansSerif => "Sans-serif",
            Family::Cursive => "Cursive",
            Family::Fantasy => "Fantasy",
            Family::Monospace => "Monospace",
        })
    }
}

/// The weight of some text.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Weight {
    Thin,
    ExtraLight,
    Light,
    #[default]
    Normal,
    Medium,
    Semibold,
    Bold,
    ExtraBold,
    Black,
}

/// An error that results from trying to convert a cosmic_text::Weight into an
/// Iced Weight.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum WeightConversionError {
    /// Iced only allows for certain predefined weight constants, and anything
    /// other than those will result in a conversion error.
    NoMatchingWeight(u16),
}

impl TryFrom<cosmic_text::Weight> for Weight {
    type Error = WeightConversionError;

    fn try_from(weight: cosmic_text::Weight) -> Result<Weight, WeightConversionError> {
        Ok(match weight {
            cosmic_text::Weight::BLACK => Weight::Thin,
            cosmic_text::Weight::EXTRA_BOLD => Weight::ExtraLight,
            cosmic_text::Weight::BOLD => Weight::Light,
            cosmic_text::Weight::SEMIBOLD => Weight::Normal,
            cosmic_text::Weight::MEDIUM => Weight::Medium,
            cosmic_text::Weight::NORMAL => Weight::Semibold,
            cosmic_text::Weight::LIGHT => Weight::Bold,
            cosmic_text::Weight::EXTRA_LIGHT => Weight::ExtraBold,
            cosmic_text::Weight::THIN => Weight::Black,
            cosmic_text::Weight(w) => return Err(WeightConversionError::NoMatchingWeight(w)),
        })
    }
}

impl From<Weight> for cosmic_text::Weight {
    fn from(weight: Weight) -> cosmic_text::Weight {
        match weight {
            Weight::Thin => cosmic_text::Weight::THIN,
            Weight::ExtraLight => cosmic_text::Weight::EXTRA_LIGHT,
            Weight::Light => cosmic_text::Weight::LIGHT,
            Weight::Normal => cosmic_text::Weight::NORMAL,
            Weight::Medium => cosmic_text::Weight::MEDIUM,
            Weight::Semibold => cosmic_text::Weight::SEMIBOLD,
            Weight::Bold => cosmic_text::Weight::BOLD,
            Weight::ExtraBold => cosmic_text::Weight::EXTRA_BOLD,
            Weight::Black => cosmic_text::Weight::BLACK,
        }
    }
}

/// The width of some text.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Stretch {
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    #[default]
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

impl From<cosmic_text::Stretch> for Stretch {
    fn from(stretch: cosmic_text::Stretch) -> Stretch {
        match stretch {
            cosmic_text::Stretch::UltraCondensed => Stretch::UltraCondensed,
            cosmic_text::Stretch::ExtraCondensed => Stretch::ExtraCondensed,
            cosmic_text::Stretch::Condensed => Stretch::Condensed,
            cosmic_text::Stretch::SemiCondensed => Stretch::SemiCondensed,
            cosmic_text::Stretch::Normal => Stretch::Normal,
            cosmic_text::Stretch::SemiExpanded => Stretch::SemiExpanded,
            cosmic_text::Stretch::Expanded => Stretch::Expanded,
            cosmic_text::Stretch::ExtraExpanded => Stretch::ExtraExpanded,
            cosmic_text::Stretch::UltraExpanded => Stretch::UltraExpanded,
        }
    }
}

impl From<Stretch> for cosmic_text::Stretch {
    fn from(stretch: Stretch) -> cosmic_text::Stretch {
        match stretch {
            Stretch::UltraCondensed => cosmic_text::Stretch::UltraCondensed,
            Stretch::ExtraCondensed => cosmic_text::Stretch::ExtraCondensed,
            Stretch::Condensed => cosmic_text::Stretch::Condensed,
            Stretch::SemiCondensed => cosmic_text::Stretch::SemiCondensed,
            Stretch::Normal => cosmic_text::Stretch::Normal,
            Stretch::SemiExpanded => cosmic_text::Stretch::SemiExpanded,
            Stretch::Expanded => cosmic_text::Stretch::Expanded,
            Stretch::ExtraExpanded => cosmic_text::Stretch::ExtraExpanded,
            Stretch::UltraExpanded => cosmic_text::Stretch::UltraExpanded,
        }
    }
}

/// The style of some text.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Style {
    #[default]
    Normal,
    Italic,
    Oblique,
}

impl From<cosmic_text::Style> for Style {
    fn from(style: cosmic_text::Style) -> Style {
        match style {
            cosmic_text::Style::Normal => Style::Normal,
            cosmic_text::Style::Italic => Style::Italic,
            cosmic_text::Style::Oblique => Style::Oblique,
        }
    }
}

impl From<Style> for cosmic_text::Style {
    fn from(style: Style) -> cosmic_text::Style {
        match style {
            Style::Normal => cosmic_text::Style::Normal,
            Style::Italic => cosmic_text::Style::Italic,
            Style::Oblique => cosmic_text::Style::Oblique,
        }
    }
}

/// A font error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {}
