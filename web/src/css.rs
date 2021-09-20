//! Style your widgets.
use crate::bumpalo;
use crate::{Alignment, Background, Color, Length, Padding};

use std::collections::BTreeMap;

/// A CSS rule of a VDOM node.
#[derive(Debug)]
pub enum Rule {
    /// Container with vertical distribution
    Column,

    /// Container with horizonal distribution
    Row,

    /// Spacing between elements
    Spacing(u16),

    /// Toggler input for a specific size
    Toggler(u16),
}

impl Rule {
    /// Returns the class name of the [`Rule`].
    pub fn class<'a>(&self) -> String {
        match self {
            Rule::Column => String::from("c"),
            Rule::Row => String::from("r"),
            Rule::Spacing(spacing) => format!("s-{}", spacing),
            Rule::Toggler(size) => format!("toggler-{}", size),
        }
    }

    /// Returns the declaration of the [`Rule`].
    pub fn declaration<'a>(&self, bump: &'a bumpalo::Bump) -> &'a str {
        let class = self.class();

        match self {
            Rule::Column => {
                let body = "{ display: flex; flex-direction: column; }";

                bumpalo::format!(in bump, ".{} {}", class, body).into_bump_str()
            }
            Rule::Row => {
                let body = "{ display: flex; flex-direction: row; }";

                bumpalo::format!(in bump, ".{} {}", class, body).into_bump_str()
            }
            Rule::Spacing(spacing) => bumpalo::format!(
                in bump,
                ".c.{} > * {{ margin-bottom: {}px }} \
                 .r.{} > * {{ margin-right: {}px }} \
                 .c.{} > *:last-child {{ margin-bottom: 0 }} \
                 .r.{} > *:last-child {{ margin-right: 0 }}",
                class,
                spacing,
                class,
                spacing,
                class,
                class
            )
            .into_bump_str(),
            Rule::Toggler(size) => bumpalo::format!(
                in bump,
                ".toggler-{} {{ display: flex; cursor: pointer; justify-content: space-between; }} \
                 .toggler-{} input {{ display:none; }} \
                 .toggler-{} span {{ background-color: #b1b1b1; position: relative; display: inline-flex; width:{}px; height: {}px; border-radius: {}px;}} \
                 .toggler-{} span > span {{ background-color: #FFFFFF; width: {}px; height: {}px; border-radius: 50%; top: 1px; left: 1px;}} \
                 .toggler-{}:hover span > span {{ background-color: #f1f1f1 !important; }} \
                 .toggler-{} input:checked + span {{ background-color: #00FF00; }} \
                 .toggler-{} input:checked + span > span {{ -webkit-transform: translateX({}px); -ms-transform:translateX({}px); transform: translateX({}px); }}
                ",
                // toggler
                size,

                // toggler input
                size,

                // toggler span
                size,
                size*2,
                size,
                size,

                // toggler span > span
                size,
                size-2,
                size-2,

                // toggler: hover + span > span
                size,

                // toggler input:checked + span
                size,

                // toggler input:checked + span > span
                size,
                size,
                size,
                size
            )
            .into_bump_str(),
        }
    }
}

/// A cascading style sheet.
#[derive(Debug)]
pub struct Css<'a> {
    rules: BTreeMap<String, &'a str>,
}

impl<'a> Css<'a> {
    /// Creates an empty [`Css`].
    pub fn new() -> Self {
        Css {
            rules: BTreeMap::new(),
        }
    }

    /// Inserts the [`Rule`] in the [`Css`], if it was not previously
    /// inserted.
    ///
    /// It returns the class name of the provided [`Rule`].
    pub fn insert(&mut self, bump: &'a bumpalo::Bump, rule: Rule) -> String {
        let class = rule.class();

        if !self.rules.contains_key(&class) {
            let _ = self.rules.insert(class.clone(), rule.declaration(bump));
        }

        class
    }

    /// Produces the VDOM node of the [`Css`].
    pub fn node(self, bump: &'a bumpalo::Bump) -> dodrio::Node<'a> {
        use dodrio::builder::*;

        let mut declarations = bumpalo::collections::Vec::new_in(bump);

        declarations.push(text("html { height: 100% }"));
        declarations.push(text(
            "body { height: 100%; margin: 0; padding: 0; font-family: sans-serif }",
        ));
        declarations.push(text("* { margin: 0; padding: 0 }"));
        declarations.push(text(
            "button { border: none; cursor: pointer; outline: none }",
        ));

        for declaration in self.rules.values() {
            declarations.push(text(*declaration));
        }

        style(bump).children(declarations).finish()
    }
}

/// Returns the style value for the given [`Length`].
pub fn length(length: Length) -> String {
    match length {
        Length::Shrink => String::from("auto"),
        Length::Units(px) => format!("{}px", px),
        Length::Fill | Length::FillPortion(_) => String::from("100%"),
    }
}

/// Returns the style value for the given maximum length in units.
pub fn max_length(units: u32) -> String {
    use std::u32;

    if units == u32::MAX {
        String::from("initial")
    } else {
        format!("{}px", units)
    }
}

/// Returns the style value for the given minimum length in units.
pub fn min_length(units: u32) -> String {
    if units == 0 {
        String::from("initial")
    } else {
        format!("{}px", units)
    }
}

/// Returns the style value for the given [`Color`].
pub fn color(Color { r, g, b, a }: Color) -> String {
    format!("rgba({}, {}, {}, {})", 255.0 * r, 255.0 * g, 255.0 * b, a)
}

/// Returns the style value for the given [`Background`].
pub fn background(background: Background) -> String {
    match background {
        Background::Color(c) => color(c),
    }
}

/// Returns the style value for the given [`Alignment`].
pub fn alignment(alignment: Alignment) -> &'static str {
    match alignment {
        Alignment::Start => "flex-start",
        Alignment::Center => "center",
        Alignment::End => "flex-end",
        Alignment::Fill => "stretch",
    }
}

/// Returns the style value for the given [`Padding`].
///
/// [`Padding`]: struct.Padding.html
pub fn padding(padding: Padding) -> String {
    format!(
        "{}px {}px {}px {}px",
        padding.top, padding.right, padding.bottom, padding.left
    )
}
