//! Style your widgets.
use crate::{bumpalo, Align, Color, Length};

use std::collections::BTreeMap;

/// The style of a VDOM node.
#[derive(Debug)]
pub enum Style {
    /// Container with vertical distribution
    Column,

    /// Container with horizonal distribution
    Row,

    /// Padding of the container
    Padding(u16),

    /// Spacing between elements
    Spacing(u16),
}

impl Style {
    /// Returns the class name of the [`Style`].
    ///
    /// [`Style`]: enum.Style.html
    pub fn class<'a>(&self) -> String {
        match self {
            Style::Column => String::from("c"),
            Style::Row => String::from("r"),
            Style::Padding(padding) => format!("p-{}", padding),
            Style::Spacing(spacing) => format!("s-{}", spacing),
        }
    }

    /// Returns the declaration of the [`Style`].
    ///
    /// [`Style`]: enum.Style.html
    pub fn declaration<'a>(&self, bump: &'a bumpalo::Bump) -> &'a str {
        let class = self.class();

        match self {
            Style::Column => {
                let body = "{ display: flex; flex-direction: column; }";

                bumpalo::format!(in bump, ".{} {}", class, body).into_bump_str()
            }
            Style::Row => {
                let body = "{ display: flex; flex-direction: row; }";

                bumpalo::format!(in bump, ".{} {}", class, body).into_bump_str()
            }
            Style::Padding(padding) => bumpalo::format!(
                in bump,
                ".{} {{ box-sizing: border-box; padding: {}px }}",
                class,
                padding
            )
            .into_bump_str(),
            Style::Spacing(spacing) => bumpalo::format!(
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

/// A sheet of styles.
#[derive(Debug)]
pub struct Sheet<'a> {
    styles: BTreeMap<String, &'a str>,
}

impl<'a> Sheet<'a> {
    /// Creates an empty style [`Sheet`].
    ///
    /// [`Sheet`]: struct.Sheet.html
    pub fn new() -> Self {
        Self {
            styles: BTreeMap::new(),
        }
    }

    /// Inserts the [`Style`] in the [`Sheet`], if it was not previously
    /// inserted.
    ///
    /// It returns the class name of the provided [`Style`].
    ///
    /// [`Sheet`]: struct.Sheet.html
    /// [`Style`]: enum.Style.html
    pub fn insert(&mut self, bump: &'a bumpalo::Bump, style: Style) -> String {
        let class = style.class();

        if !self.styles.contains_key(&class) {
            let _ = self.styles.insert(class.clone(), style.declaration(bump));
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
        declarations.push(text("p { margin: 0 }"));
        declarations.push(text(
            "button { border: none; cursor: pointer; outline: none }",
        ));

        for declaration in self.styles.values() {
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

/// Returns the style value for the given [`Color`].
///
/// [`Color`]: ../struct.Color.html
pub fn color(Color { r, g, b, a }: Color) -> String {
    format!("rgba({}, {}, {}, {})", 255.0 * r, 255.0 * g, 255.0 * b, a)
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
