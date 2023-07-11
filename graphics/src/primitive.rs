//! Draw using different graphical primitives.
use crate::core::alignment;
use crate::core::image;
use crate::core::renderer::Effect;
use crate::core::svg;
use crate::core::text;
use crate::core::{Background, Color, Font, Rectangle, Vector};

use std::sync::Arc;

/// A rendering primitive.
#[derive(Debug, Clone, PartialEq)]
pub enum Primitive<T> {
    /// A text primitive
    Text {
        /// The contents of the text
        content: String,
        /// The bounds of the text
        bounds: Rectangle,
        /// The color of the text
        color: Color,
        /// The size of the text in logical pixels
        size: f32,
        /// The line height of the text
        line_height: text::LineHeight,
        /// The font of the text
        font: Font,
        /// The horizontal alignment of the text
        horizontal_alignment: alignment::Horizontal,
        /// The vertical alignment of the text
        vertical_alignment: alignment::Vertical,
        /// The shaping strategy of the text.
        shaping: text::Shaping,
    },
    /// A quad primitive
    Quad {
        /// The bounds of the quad
        bounds: Rectangle,
        /// The background of the quad
        background: Background,
        /// The border radii of the quad
        border_radius: [f32; 4],
        /// The border width of the quad
        border_width: f32,
        /// The border color of the quad
        border_color: Color,
    },
    /// An image primitive
    Image {
        /// The handle of the image
        handle: image::Handle,
        /// The bounds of the image
        bounds: Rectangle,
    },
    /// An SVG primitive
    Svg {
        /// The path of the SVG file
        handle: svg::Handle,

        /// The [`Color`] filter
        color: Option<Color>,

        /// The bounds of the viewport
        bounds: Rectangle,
    },
    /// A group of primitives
    Group {
        /// The primitives of the group
        primitives: Vec<Primitive<T>>,
    },
    /// A clip primitive
    Clip {
        /// The bounds of the clip
        bounds: Rectangle,
        /// The content of the clip
        content: Box<Primitive<T>>,
    },
    /// A primitive that applies a translation
    Translate {
        /// The translation vector
        translation: Vector,

        /// The primitive to translate
        content: Box<Primitive<T>>,
    },
    /// A cached primitive.
    ///
    /// This can be useful if you are implementing a widget where primitive
    /// generation is expensive.
    Cache {
        /// The cached primitive
        content: Arc<Primitive<T>>,
    },
    /// A primitive which can apply [`Effect`]s to other primitives.
    Effect {
        /// The bounds of the [`Effect`].
        bounds: Rectangle,
        /// The [`Effect`] to apply to the `content`.
        effect: Effect,
        /// The primitives to apply the [`Effect`] to.
        content: Box<Primitive<T>>,
    },
    /// A backend-specific primitive.
    Custom(T),
}

impl<T> Primitive<T> {
    /// Creates a [`Primitive::Group`].
    pub fn group(primitives: Vec<Self>) -> Self {
        Self::Group { primitives }
    }

    /// Creates a [`Primitive::Clip`].
    pub fn clip(self, bounds: Rectangle) -> Self {
        Self::Clip {
            bounds,
            content: Box::new(self),
        }
    }

    /// Creates a [`Primitive::Translate`].
    pub fn translate(self, translation: Vector) -> Self {
        Self::Translate {
            translation,
            content: Box::new(self),
        }
    }

    /// Wraps a primitive in an [`Effect`].
    pub fn effect(self, bounds: Rectangle, effect: Effect) -> Self {
        Self::Effect {
            bounds,
            effect,
            content: Box::new(self),
        }
    }
}
