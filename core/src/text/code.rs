//! Primitives for parsing code and highlighting.
use crate::Theme;
use crate::font;
use crate::text::highlighter::{self, Highlighter};

/// A specific region of code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Code {
    /// A comment.
    Comment,
    /// A string or character literal.
    String,
    /// A keyword or storage word.
    Keyword,
    /// A constant, numeric, or boolean literal.
    Constant,
    /// A function or method name.
    Function,
    /// A type, class, or tag name.
    Type,
    /// A variable.
    Variable,
    /// A built-in or support symbol.
    Support,
    /// Punctuation.
    Punctuation,
    /// A path component.
    Path,
    /// An invalid or erroneous construct.
    Invalid,
    /// Anything that does not match another class.
    Other,
}

impl Highlighter<Code> for Theme {
    fn highlight(&self, code: Code) -> highlighter::Style {
        let palette = self.palette();

        let color = match code {
            Code::Keyword => Some(palette.primary.base.color),
            Code::Type | Code::Path | Code::Function => Some(palette.warning.base.color),

            Code::Variable => Some(palette.danger.base.color),
            Code::Constant => Some(palette.danger.base.color),
            Code::String => Some(palette.success.base.color),
            Code::Support => Some(palette.primary.base.color),

            Code::Punctuation => Some(palette.secondary.strong.color),
            Code::Comment => Some(palette.secondary.base.color),

            Code::Invalid => Some(palette.danger.base.color),
            Code::Other => None,
        };

        highlighter::Style {
            color,
            style: (code == Code::Comment).then_some(font::Style::Italic),
        }
    }
}
