//! A syntax highlighter for iced.
use iced_core as core;

use crate::core::Color;
use crate::core::font::{self, Font};
use crate::core::text::highlighter::{self, Format};

use std::ops::Range;
use std::sync::LazyLock;

use syntect::highlighting;
use syntect::parsing;

static SYNTAXES: LazyLock<parsing::SyntaxSet> =
    LazyLock::new(parsing::SyntaxSet::load_defaults_nonewlines);

static THEMES: LazyLock<highlighting::ThemeSet> =
    LazyLock::new(highlighting::ThemeSet::load_defaults);

const LINES_PER_SNAPSHOT: usize = 50;

/// A syntax highlighter.
#[derive(Debug)]
pub struct Highlighter {
    syntax: &'static parsing::SyntaxReference,
    highlighter: highlighting::Highlighter<'static>,
    caches: Vec<(parsing::ParseState, parsing::ScopeStack)>,
    current_line: usize,
}

impl highlighter::Highlighter for Highlighter {
    type Settings = Settings;
    type Highlight = Highlight;

    type Iterator<'a> =
        Box<dyn Iterator<Item = (Range<usize>, Self::Highlight)> + 'a>;

    fn new(settings: &Self::Settings) -> Self {
        let syntax = SYNTAXES
            .find_syntax_by_token(&settings.token)
            .unwrap_or_else(|| SYNTAXES.find_syntax_plain_text());

        let highlighter = highlighting::Highlighter::new(
            &THEMES.themes[settings.theme.key()],
        );

        let parser = parsing::ParseState::new(syntax);
        let stack = parsing::ScopeStack::new();

        Highlighter {
            syntax,
            highlighter,
            caches: vec![(parser, stack)],
            current_line: 0,
        }
    }

    fn update(&mut self, new_settings: &Self::Settings) {
        self.syntax = SYNTAXES
            .find_syntax_by_token(&new_settings.token)
            .unwrap_or_else(|| SYNTAXES.find_syntax_plain_text());

        self.highlighter = highlighting::Highlighter::new(
            &THEMES.themes[new_settings.theme.key()],
        );

        // Restart the highlighter
        self.change_line(0);
    }

    fn change_line(&mut self, line: usize) {
        let snapshot = line / LINES_PER_SNAPSHOT;

        if snapshot <= self.caches.len() {
            self.caches.truncate(snapshot);
            self.current_line = snapshot * LINES_PER_SNAPSHOT;
        } else {
            self.caches.truncate(1);
            self.current_line = 0;
        }

        let (parser, stack) =
            self.caches.last().cloned().unwrap_or_else(|| {
                (
                    parsing::ParseState::new(self.syntax),
                    parsing::ScopeStack::new(),
                )
            });

        self.caches.push((parser, stack));
    }

    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        if self.current_line / LINES_PER_SNAPSHOT >= self.caches.len() {
            let (parser, stack) =
                self.caches.last().expect("Caches must not be empty");

            self.caches.push((parser.clone(), stack.clone()));
        }

        self.current_line += 1;

        let (parser, stack) =
            self.caches.last_mut().expect("Caches must not be empty");

        let ops = parser.parse_line(line, &SYNTAXES).unwrap_or_default();

        Box::new(scope_iterator(ops, line, stack, &self.highlighter))
    }

    fn current_line(&self) -> usize {
        self.current_line
    }
}

fn scope_iterator<'a>(
    ops: Vec<(usize, parsing::ScopeStackOp)>,
    line: &str,
    stack: &'a mut parsing::ScopeStack,
    highlighter: &'a highlighting::Highlighter<'static>,
) -> impl Iterator<Item = (Range<usize>, Highlight)> + 'a {
    ScopeRangeIterator {
        ops,
        line_length: line.len(),
        index: 0,
        last_str_index: 0,
    }
    .filter_map(move |(range, scope)| {
        let _ = stack.apply(&scope);

        if range.is_empty() {
            None
        } else {
            Some((
                range,
                Highlight(highlighter.style_mod_for_stack(&stack.scopes)),
            ))
        }
    })
}

/// A streaming syntax highlighter.
///
/// It can efficiently highlight an immutable stream of tokens.
#[derive(Debug)]
pub struct Stream {
    syntax: &'static parsing::SyntaxReference,
    highlighter: highlighting::Highlighter<'static>,
    commit: (parsing::ParseState, parsing::ScopeStack),
    state: parsing::ParseState,
    stack: parsing::ScopeStack,
}

impl Stream {
    /// Creates a new [`Stream`] highlighter.
    pub fn new(settings: &Settings) -> Self {
        let syntax = SYNTAXES
            .find_syntax_by_token(&settings.token)
            .unwrap_or_else(|| SYNTAXES.find_syntax_plain_text());

        let highlighter = highlighting::Highlighter::new(
            &THEMES.themes[settings.theme.key()],
        );

        let state = parsing::ParseState::new(syntax);
        let stack = parsing::ScopeStack::new();

        Self {
            syntax,
            highlighter,
            commit: (state.clone(), stack.clone()),
            state,
            stack,
        }
    }

    /// Highlights the given line from the last commit.
    pub fn highlight_line(
        &mut self,
        line: &str,
    ) -> impl Iterator<Item = (Range<usize>, Highlight)> + '_ {
        self.state = self.commit.0.clone();
        self.stack = self.commit.1.clone();

        let ops = self.state.parse_line(line, &SYNTAXES).unwrap_or_default();
        scope_iterator(ops, line, &mut self.stack, &self.highlighter)
    }

    /// Commits the last highlighted line.
    pub fn commit(&mut self) {
        self.commit = (self.state.clone(), self.stack.clone());
    }

    /// Resets the [`Stream`] highlighter.
    pub fn reset(&mut self) {
        self.state = parsing::ParseState::new(self.syntax);
        self.stack = parsing::ScopeStack::new();
        self.commit = (self.state.clone(), self.stack.clone());
    }
}

/// The settings of a [`Highlighter`].
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    /// The [`Theme`] of the [`Highlighter`].
    ///
    /// It dictates the color scheme that will be used for highlighting.
    pub theme: Theme,
    /// The extension of the file or the name of the language to highlight.
    ///
    /// The [`Highlighter`] will use the token to automatically determine
    /// the grammar to use for highlighting.
    pub token: String,
}

/// A highlight produced by a [`Highlighter`].
#[derive(Debug)]
pub struct Highlight(highlighting::StyleModifier);

impl Highlight {
    /// Returns the color of this [`Highlight`].
    ///
    /// If `None`, the original text color should be unchanged.
    pub fn color(&self) -> Option<Color> {
        self.0.foreground.map(|color| {
            Color::from_rgba8(color.r, color.g, color.b, color.a as f32 / 255.0)
        })
    }

    /// Returns the font of this [`Highlight`].
    ///
    /// If `None`, the original font should be unchanged.
    pub fn font(&self) -> Option<Font> {
        self.0.font_style.and_then(|style| {
            let bold = style.contains(highlighting::FontStyle::BOLD);
            let italic = style.contains(highlighting::FontStyle::ITALIC);

            if bold || italic {
                Some(Font {
                    weight: if bold {
                        font::Weight::Bold
                    } else {
                        font::Weight::Normal
                    },
                    style: if italic {
                        font::Style::Italic
                    } else {
                        font::Style::Normal
                    },
                    ..Font::MONOSPACE
                })
            } else {
                None
            }
        })
    }

    /// Returns the [`Format`] of the [`Highlight`].
    ///
    /// It contains both the [`color`] and the [`font`].
    ///
    /// [`color`]: Self::color
    /// [`font`]: Self::font
    pub fn to_format(&self) -> Format<Font> {
        Format {
            color: self.color(),
            font: self.font(),
        }
    }
}

/// A highlighting theme.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    SolarizedDark,
    Base16Mocha,
    Base16Ocean,
    Base16Eighties,
    InspiredGitHub,
}

impl Theme {
    /// A static slice containing all the available themes.
    pub const ALL: &'static [Self] = &[
        Self::SolarizedDark,
        Self::Base16Mocha,
        Self::Base16Ocean,
        Self::Base16Eighties,
        Self::InspiredGitHub,
    ];

    /// Returns `true` if the [`Theme`] is dark, and false otherwise.
    pub fn is_dark(self) -> bool {
        match self {
            Self::SolarizedDark
            | Self::Base16Mocha
            | Self::Base16Ocean
            | Self::Base16Eighties => true,
            Self::InspiredGitHub => false,
        }
    }

    fn key(self) -> &'static str {
        match self {
            Theme::SolarizedDark => "Solarized (dark)",
            Theme::Base16Mocha => "base16-mocha.dark",
            Theme::Base16Ocean => "base16-ocean.dark",
            Theme::Base16Eighties => "base16-eighties.dark",
            Theme::InspiredGitHub => "InspiredGitHub",
        }
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::SolarizedDark => write!(f, "Solarized Dark"),
            Theme::Base16Mocha => write!(f, "Mocha"),
            Theme::Base16Ocean => write!(f, "Ocean"),
            Theme::Base16Eighties => write!(f, "Eighties"),
            Theme::InspiredGitHub => write!(f, "Inspired GitHub"),
        }
    }
}

struct ScopeRangeIterator {
    ops: Vec<(usize, parsing::ScopeStackOp)>,
    line_length: usize,
    index: usize,
    last_str_index: usize,
}

impl Iterator for ScopeRangeIterator {
    type Item = (std::ops::Range<usize>, parsing::ScopeStackOp);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index > self.ops.len() {
            return None;
        }

        let next_str_i = if self.index == self.ops.len() {
            self.line_length
        } else {
            self.ops[self.index].0
        };

        let range = self.last_str_index..next_str_i;
        self.last_str_index = next_str_i;

        let op = if self.index == 0 {
            parsing::ScopeStackOp::Noop
        } else {
            self.ops[self.index - 1].1.clone()
        };

        self.index += 1;
        Some((range, op))
    }
}
