use crate::core::widget::Id;
use crate::core::widget::operation::{Focusable, Scrollable, TextInput};
use crate::core::{Rectangle, Vector};

use std::any::Any;

#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    Container {
        id: Option<Id>,
        bounds: Rectangle,
        visible_bounds: Option<Rectangle>,
    },
    Focusable {
        id: Option<Id>,
        bounds: Rectangle,
        visible_bounds: Option<Rectangle>,
    },
    Scrollable {
        id: Option<Id>,
        bounds: Rectangle,
        visible_bounds: Option<Rectangle>,
        content_bounds: Rectangle,
        translation: Vector,
    },
    TextInput {
        id: Option<Id>,
        bounds: Rectangle,
        visible_bounds: Option<Rectangle>,
        content: String,
    },
    Text {
        id: Option<Id>,
        bounds: Rectangle,
        visible_bounds: Option<Rectangle>,
        content: String,
    },
    Custom {
        id: Option<Id>,
        bounds: Rectangle,
        visible_bounds: Option<Rectangle>,
    },
}

impl Target {
    pub fn bounds(&self) -> Rectangle {
        match self {
            Target::Container { bounds, .. }
            | Target::Focusable { bounds, .. }
            | Target::Scrollable { bounds, .. }
            | Target::TextInput { bounds, .. }
            | Target::Text { bounds, .. }
            | Target::Custom { bounds, .. } => *bounds,
        }
    }

    pub fn visible_bounds(&self) -> Option<Rectangle> {
        match self {
            Target::Container { visible_bounds, .. }
            | Target::Focusable { visible_bounds, .. }
            | Target::Scrollable { visible_bounds, .. }
            | Target::TextInput { visible_bounds, .. }
            | Target::Text { visible_bounds, .. }
            | Target::Custom { visible_bounds, .. } => *visible_bounds,
        }
    }
}

impl From<Candidate<'_>> for Target {
    fn from(candidate: Candidate<'_>) -> Self {
        match candidate {
            Candidate::Container {
                id,
                bounds,
                visible_bounds,
            } => Self::Container {
                id: id.cloned(),
                bounds,
                visible_bounds,
            },
            Candidate::Focusable {
                id,
                bounds,
                visible_bounds,
                ..
            } => Self::Focusable {
                id: id.cloned(),
                bounds,
                visible_bounds,
            },
            Candidate::Scrollable {
                id,
                bounds,
                visible_bounds,
                content_bounds,
                translation,
                ..
            } => Self::Scrollable {
                id: id.cloned(),
                bounds,
                visible_bounds,
                content_bounds,
                translation,
            },
            Candidate::TextInput {
                id,
                bounds,
                visible_bounds,
                state,
            } => Self::TextInput {
                id: id.cloned(),
                bounds,
                visible_bounds,
                content: state.text().to_owned(),
            },
            Candidate::Text {
                id,
                bounds,
                visible_bounds,
                content,
            } => Self::Text {
                id: id.cloned(),
                bounds,
                visible_bounds,
                content: content.to_owned(),
            },
            Candidate::Custom {
                id,
                bounds,
                visible_bounds,
                ..
            } => Self::Custom {
                id: id.cloned(),
                bounds,
                visible_bounds,
            },
        }
    }
}

impl Bounded for Target {
    fn bounds(&self) -> Rectangle {
        self.bounds()
    }

    fn visible_bounds(&self) -> Option<Rectangle> {
        self.visible_bounds()
    }
}

#[derive(Clone)]
pub enum Candidate<'a> {
    Container {
        id: Option<&'a Id>,
        bounds: Rectangle,
        visible_bounds: Option<Rectangle>,
    },
    Focusable {
        id: Option<&'a Id>,
        bounds: Rectangle,
        visible_bounds: Option<Rectangle>,
        state: &'a dyn Focusable,
    },
    Scrollable {
        id: Option<&'a Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        visible_bounds: Option<Rectangle>,
        translation: Vector,
        state: &'a dyn Scrollable,
    },
    TextInput {
        id: Option<&'a Id>,
        bounds: Rectangle,
        visible_bounds: Option<Rectangle>,
        state: &'a dyn TextInput,
    },
    Text {
        id: Option<&'a Id>,
        bounds: Rectangle,
        visible_bounds: Option<Rectangle>,
        content: &'a str,
    },
    Custom {
        id: Option<&'a Id>,
        bounds: Rectangle,
        visible_bounds: Option<Rectangle>,
        state: &'a dyn Any,
    },
}

impl<'a> Candidate<'a> {
    pub fn id(&self) -> Option<&'a Id> {
        match self {
            Candidate::Container { id, .. }
            | Candidate::Focusable { id, .. }
            | Candidate::Scrollable { id, .. }
            | Candidate::TextInput { id, .. }
            | Candidate::Text { id, .. }
            | Candidate::Custom { id, .. } => *id,
        }
    }

    pub fn bounds(&self) -> Rectangle {
        match self {
            Candidate::Container { bounds, .. }
            | Candidate::Focusable { bounds, .. }
            | Candidate::Scrollable { bounds, .. }
            | Candidate::TextInput { bounds, .. }
            | Candidate::Text { bounds, .. }
            | Candidate::Custom { bounds, .. } => *bounds,
        }
    }

    pub fn visible_bounds(&self) -> Option<Rectangle> {
        match self {
            Candidate::Container { visible_bounds, .. }
            | Candidate::Focusable { visible_bounds, .. }
            | Candidate::Scrollable { visible_bounds, .. }
            | Candidate::TextInput { visible_bounds, .. }
            | Candidate::Text { visible_bounds, .. }
            | Candidate::Custom { visible_bounds, .. } => *visible_bounds,
        }
    }
}

pub trait Bounded: std::fmt::Debug {
    fn bounds(&self) -> Rectangle;

    fn visible_bounds(&self) -> Option<Rectangle>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Text {
    Raw {
        id: Option<Id>,
        bounds: Rectangle,
        visible_bounds: Option<Rectangle>,
    },
    Input {
        id: Option<Id>,
        bounds: Rectangle,
        visible_bounds: Option<Rectangle>,
    },
}

impl Text {
    pub fn bounds(&self) -> Rectangle {
        match self {
            Text::Raw { bounds, .. } | Text::Input { bounds, .. } => *bounds,
        }
    }

    pub fn visible_bounds(&self) -> Option<Rectangle> {
        match self {
            Text::Raw { visible_bounds, .. }
            | Text::Input { visible_bounds, .. } => *visible_bounds,
        }
    }
}

impl Bounded for Text {
    fn bounds(&self) -> Rectangle {
        self.bounds()
    }

    fn visible_bounds(&self) -> Option<Rectangle> {
        self.visible_bounds()
    }
}
