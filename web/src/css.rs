//! Style your widgets.
use crate::{bumpalo, Align, Background, Color, Length};

use std::collections::BTreeMap;

/// A CSS rule of a VDOM node.
#[derive(Debug)]
pub enum Rule {
    /// Container with vertical distribution
    Column,

    /// Container with horizonal distribution
    Row,

    /// Padding of the container
    Padding(u16),

    /// Spacing between elements
    Spacing(u16),
}

impl Rule {
    /// Returns the class name of the [`Style`].
    ///
    /// [`Style`]: enum.Style.html
    pub fn class<'a>(&self) -> String {
        match self {
            Rule::Column => String::from("c"),
            Rule::Row => String::from("r"),
            Rule::Padding(padding) => format!("p-{}", padding),
            Rule::Spacing(spacing) => format!("s-{}", spacing),
        }
    }

    /// Returns the declaration of the [`Style`].
    ///
    /// [`Style`]: enum.Style.html
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
            Rule::Padding(padding) => bumpalo::format!(
                in bump,
                ".{} {{ box-sizing: border-box; padding: {}px }}",
                class,
                padding
            )
            .into_bump_str(),
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
        }
    }
}

/// A cascading style sheet.
#[derive(Debug)]
pub struct Css<'a> {
    rules: BTreeMap<String, &'a str>,
}

impl<'a> Css<'a> {
    /// Creates an empty style [`Sheet`].
    ///
    /// [`Sheet`]: struct.Sheet.html
    pub fn new() -> Self {
        Css {
            rules: BTreeMap::new(),
        }
    }

    /// Inserts the [`rule`] in the [`Sheet`], if it was not previously
    /// inserted.
    ///
    /// It returns the class name of the provided [`Rule`].
    ///
    /// [`Sheet`]: struct.Sheet.html
    /// [`Rule`]: enum.Rule.html
    pub fn insert(&mut self, bump: &'a bumpalo::Bump, rule: Rule) -> String {
        let class = rule.class();

        if !self.rules.contains_key(&class) {
            let _ = self.rules.insert(class.clone(), rule.declaration(bump));
        }

        class
    }

    /// Produces the VDOM node of the style [`Sheet`].
    ///
    /// [`Sheet`]: struct.Sheet.html
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
///
/// [`Length`]: ../enum.Length.html
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
///
/// [`Color`]: ../struct.Color.html
pub fn color(Color { r, g, b, a }: Color) -> String {
    format!("rgba({}, {}, {}, {})", 255.0 * r, 255.0 * g, 255.0 * b, a)
}

/// Returns the style value for the given [`Background`].
///
/// [`Background`]: ../struct.Background.html
pub fn background(background: Background) -> String {
    match background {
        Background::Color(c) => color(c),
    }
}

/// Returns the style value for the given [`Align`].
///
/// [`Align`]: ../enum.Align.html
pub fn align(align: Align) -> &'static str {
    match align {
        Align::Start => "flex-start",
        Align::Center => "center",
        Align::End => "flex-end",
    }
}
