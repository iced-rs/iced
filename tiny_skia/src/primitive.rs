use crate::{Rectangle, Vector};

use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Primitive {
    /// A group of primitives
    Group {
        /// The primitives of the group
        primitives: Vec<Primitive>,
    },
    /// A clip primitive
    Clip {
        /// The bounds of the clip
        bounds: Rectangle,
        /// The content of the clip
        content: Box<Primitive>,
    },
    /// A primitive that applies a translation
    Translate {
        /// The translation vector
        translation: Vector,

        /// The primitive to translate
        content: Box<Primitive>,
    },
    /// A cached primitive.
    ///
    /// This can be useful if you are implementing a widget where primitive
    /// generation is expensive.
    Cached {
        /// The cached primitive
        cache: Arc<Primitive>,
    },
    /// A basic primitive.
    Basic(iced_graphics::Primitive),
}

impl iced_graphics::backend::Primitive for Primitive {
    fn translate(self, translation: Vector) -> Self {
        Self::Translate {
            translation,
            content: Box::new(self),
        }
    }

    fn clip(self, bounds: Rectangle) -> Self {
        Self::Clip {
            bounds,
            content: Box::new(self),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Recording(pub(crate) Vec<Primitive>);

impl iced_graphics::backend::Recording for Recording {
    type Primitive = Primitive;

    fn push(&mut self, primitive: Primitive) {
        self.0.push(primitive);
    }

    fn push_basic(&mut self, basic: iced_graphics::Primitive) {
        self.0.push(Primitive::Basic(basic));
    }

    fn group(self) -> Self::Primitive {
        Primitive::Group { primitives: self.0 }
    }

    fn clear(&mut self) {
        self.0.clear();
    }
}

impl Recording {
    pub fn primitives(&self) -> &[Primitive] {
        &self.0
    }
}
