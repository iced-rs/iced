//! A syntax highlighter for iced.
use iced_core as core;

pub use highlighter::{Format, Highlight, Scope};

use crate::core::text::highlighter;

use std::ops::Range;
use std::sync::LazyLock;

use syntect::parsing;
use two_face::re_exports::syntect;

static SYNTAXES: LazyLock<parsing::SyntaxSet> = LazyLock::new(two_face::syntax::extra_no_newlines);

const LINES_PER_SNAPSHOT: usize = 50;

/// A syntax highlighter.
#[derive(Debug)]
pub struct Highlighter {
    syntax: &'static parsing::SyntaxReference,
    caches: Vec<(parsing::ParseState, parsing::ScopeStack)>,
    current_line: usize,
}

/// An iterator over the highlighted regions of a line.
///
/// Each item is a character range within the line, paired with
/// the [`Scope`] of the region.
pub type ScopeIterator<'a> = Box<dyn Iterator<Item = (Range<usize>, Scope)> + 'a>;

impl Highlighter {
    /// Creates a new [`Highlighter`] with the given [`Settings`].
    pub fn new(settings: &Settings) -> Self {
        let syntax = SYNTAXES
            .find_syntax_by_token(&settings.token)
            .unwrap_or_else(|| SYNTAXES.find_syntax_plain_text());

        let parser = parsing::ParseState::new(syntax);
        let stack = parsing::ScopeStack::new();

        Highlighter {
            syntax,
            caches: vec![(parser, stack)],
            current_line: 0,
        }
    }

    /// Updates the highlighter with the given [`Settings`],
    /// restarting it from the first line.
    pub fn update(&mut self, new_settings: &Settings) {
        self.syntax = SYNTAXES
            .find_syntax_by_token(&new_settings.token)
            .unwrap_or_else(|| SYNTAXES.find_syntax_plain_text());

        // Restart the highlighter
        self.change_line(0);
    }

    /// Changes the line the highlighter is currently on.
    pub fn change_line(&mut self, line: usize) {
        let snapshot = line / LINES_PER_SNAPSHOT;

        if snapshot <= self.caches.len() {
            self.caches.truncate(snapshot);
            self.current_line = snapshot * LINES_PER_SNAPSHOT;
        } else {
            self.caches.truncate(1);
            self.current_line = 0;
        }

        let (parser, stack) = self.caches.last().cloned().unwrap_or_else(|| {
            (
                parsing::ParseState::new(self.syntax),
                parsing::ScopeStack::new(),
            )
        });

        self.caches.push((parser, stack));
    }

    /// Highlights the given line, returning a [`ScopeIterator`].
    pub fn highlight_line(&mut self, line: &str) -> ScopeIterator<'_> {
        if self.current_line / LINES_PER_SNAPSHOT >= self.caches.len() {
            let (parser, stack) = self.caches.last().expect("Caches must not be empty");

            self.caches.push((parser.clone(), stack.clone()));
        }

        self.current_line += 1;

        let (parser, stack) = self.caches.last_mut().expect("Caches must not be empty");
        let ops = parser.parse_line(line, &SYNTAXES).unwrap_or_default();

        Box::new(scope_iterator(ops, line, stack))
    }

    /// Returns the line the highlighter is currently on.
    pub fn current_line(&self) -> usize {
        self.current_line
    }
}

impl highlighter::Highlighter for Highlighter {
    type Settings = Settings;
    type Iterator<'a> = ScopeIterator<'a>;

    fn new(settings: &Self::Settings) -> Self {
        Self::new(settings)
    }

    fn update(&mut self, new_settings: &Self::Settings) {
        self.update(new_settings);
    }

    fn change_line(&mut self, line: usize) {
        self.change_line(line);
    }

    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        self.highlight_line(line)
    }

    fn current_line(&self) -> usize {
        self.current_line()
    }
}

fn scope_iterator<'a>(
    ops: Vec<(usize, parsing::ScopeStackOp)>,
    line: &str,
    stack: &'a mut parsing::ScopeStack,
) -> impl Iterator<Item = (Range<usize>, Scope)> + 'a {
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
            Some((range, scope_from_stack(&stack.scopes)))
        }
    })
}

/// A streaming syntax highlighter.
///
/// It can efficiently highlight an immutable stream of tokens.
#[derive(Debug)]
pub struct Stream {
    syntax: &'static parsing::SyntaxReference,
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

        let state = parsing::ParseState::new(syntax);
        let stack = parsing::ScopeStack::new();

        Self {
            syntax,
            commit: (state.clone(), stack.clone()),
            state,
            stack,
        }
    }

    /// Highlights the given line from the last commit.
    pub fn highlight_line(
        &mut self,
        line: &str,
    ) -> impl Iterator<Item = (Range<usize>, Scope)> + '_ {
        self.state = self.commit.0.clone();
        self.stack = self.commit.1.clone();

        let ops = self.state.parse_line(line, &SYNTAXES).unwrap_or_default();
        scope_iterator(ops, line, &mut self.stack)
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
    /// The extension of the file or the name of the language to highlight.
    ///
    /// The [`Highlighter`] will use the token to automatically determine
    /// the grammar to use for highlighting.
    pub token: String,
}

/// The scope families and their classes.
///
/// The families are listed most specific first, so that e.g.
/// `entity.name.function` wins over `entity.name`.
static FAMILIES: LazyLock<Vec<(parsing::Scope, Scope)>> = LazyLock::new(|| {
    [
        ("invalid", Scope::Invalid),
        ("constant", Scope::Constant),
        ("string", Scope::String),
        ("comment", Scope::Comment),
        ("keyword", Scope::Keyword),
        ("storage", Scope::Keyword),
        ("entity.name.function", Scope::Function),
        ("entity.name", Scope::Type),
        ("entity.other.inherited-class", Scope::Type),
        ("support", Scope::Support),
        ("variable", Scope::Variable),
        ("punctuation", Scope::Punctuation),
    ]
    .into_iter()
    .map(|(name, class)| {
        (
            parsing::Scope::new(name).expect("scope family is valid"),
            class,
        )
    })
    .collect()
});

/// Classifies the scope stack of a highlighted region.
///
/// The stack is walked from the most specific scope (last) to the
/// least specific (first); the first scope that matches a family
/// determines the class. If no scope matches, the region is
/// classified as [`Scope::Other`].
fn scope_from_stack(stack: &[parsing::Scope]) -> Scope {
    for scope in stack.iter().rev() {
        for (family, class) in FAMILIES.iter() {
            if family.is_prefix_of(*scope) {
                return *class;
            }
        }
    }

    Scope::Other
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a scope stack from dotted scope names.
    fn stack(names: &[&str]) -> Vec<parsing::Scope> {
        names
            .iter()
            .map(|name| parsing::Scope::new(name).unwrap())
            .collect()
    }

    #[test]
    fn scopes_are_classified_by_family() {
        assert_eq!(
            scope_from_stack(&stack(&["source.rust", "comment.line"])),
            Scope::Comment
        );
        assert_eq!(
            scope_from_stack(&stack(&["source.rust", "comment.block"])),
            Scope::Comment
        );
        assert_eq!(
            scope_from_stack(&stack(&["source.rust", "string.quoted.double"])),
            Scope::String
        );
        assert_eq!(
            scope_from_stack(&stack(&["source.rust", "keyword.control"])),
            Scope::Keyword
        );
        assert_eq!(
            scope_from_stack(&stack(&["source.rust", "keyword.operator"])),
            Scope::Keyword
        );
        assert_eq!(
            scope_from_stack(&stack(&["source.rust", "storage.type"])),
            Scope::Keyword
        );
        assert_eq!(
            scope_from_stack(&stack(&["source.rust", "constant.numeric"])),
            Scope::Constant
        );
        assert_eq!(
            scope_from_stack(&stack(&["source.rust", "entity.name.function"])),
            Scope::Function
        );
        assert_eq!(
            scope_from_stack(&stack(&["source.rust", "entity.name.type"])),
            Scope::Type
        );
        assert_eq!(
            scope_from_stack(&stack(&["source.rust", "entity.other.inherited-class"])),
            Scope::Type
        );
        assert_eq!(
            scope_from_stack(&stack(&["source.rust", "variable.parameter"])),
            Scope::Variable
        );
        assert_eq!(
            scope_from_stack(&stack(&["source.rust", "support.function.builtin"])),
            Scope::Support
        );
        assert_eq!(
            scope_from_stack(&stack(&["source.rust", "punctuation.definition"])),
            Scope::Punctuation
        );
        assert_eq!(
            scope_from_stack(&stack(&["source.rust", "invalid.illegal"])),
            Scope::Invalid
        );
    }

    #[test]
    fn the_most_specific_scope_wins() {
        // An escape sequence inside a string is a constant, not a string.
        let names = [
            "source.rust",
            "string.quoted.double",
            "constant.character.escape",
        ];

        assert_eq!(scope_from_stack(&stack(&names)), Scope::Constant);

        // A comment marker is punctuation, even inside a comment.
        let names = [
            "source.rust",
            "comment.line",
            "punctuation.definition.comment",
        ];

        assert_eq!(scope_from_stack(&stack(&names)), Scope::Punctuation);
    }

    #[test]
    fn the_walk_continues_to_shallower_scopes() {
        // The leaf scope is unmatched, so the classification falls back
        // to the next scope in the stack.
        let names = ["source.rust", "string.quoted.double", "meta.embedded"];

        assert_eq!(scope_from_stack(&stack(&names)), Scope::String);
    }

    #[test]
    fn unmatched_scopes_are_other() {
        assert_eq!(scope_from_stack(&[]), Scope::Other);
        assert_eq!(
            scope_from_stack(&stack(&["source.rust", "meta.function"])),
            Scope::Other
        );

        // A family must match whole scope atoms: `stringify` is not a
        // `string`.
        assert_eq!(
            scope_from_stack(&stack(&["source.rust", "stringify.call"])),
            Scope::Other
        );
    }
}
