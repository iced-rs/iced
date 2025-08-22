use crate::core::widget::Id;
use crate::core::widget::operation::{Focusable, Scrollable, TextInput};
use crate::core::{Rectangle, Vector};

use std::any::Any;

#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub enum Target<'a> {
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

impl<'a> Target<'a> {
    pub fn id(&self) -> Option<&'a Id> {
        match self {
            Target::Container { id, .. }
            | Target::Focusable { id, .. }
            | Target::Scrollable { id, .. }
            | Target::TextInput { id, .. }
            | Target::Text { id, .. }
            | Target::Custom { id, .. } => *id,
        }
    }

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
}

#[derive(Debug, Clone, PartialEq)]
pub enum Match {
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

impl Match {
    pub fn from_target(target: Target<'_>) -> Self {
        match target {
            Target::Container {
                id,
                bounds,
                visible_bounds,
            } => Self::Container {
                id: id.cloned(),
                bounds,
                visible_bounds,
            },
            Target::Focusable {
                id,
                bounds,
                visible_bounds,
                ..
            } => Self::Focusable {
                id: id.cloned(),
                bounds,
                visible_bounds,
            },
            Target::Scrollable {
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
            Target::TextInput {
                id,
                bounds,
                visible_bounds,
                ..
            } => Self::TextInput {
                id: id.cloned(),
                bounds,
                visible_bounds,
            },
            Target::Text {
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
            Target::Custom {
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

impl Bounded for Match {
    fn bounds(&self) -> Rectangle {
        match self {
            Match::Container { bounds, .. }
            | Match::Focusable { bounds, .. }
            | Match::Scrollable { bounds, .. }
            | Match::TextInput { bounds, .. }
            | Match::Text { bounds, .. }
            | Match::Custom { bounds, .. } => *bounds,
        }
    }

    fn visible_bounds(&self) -> Option<Rectangle> {
        match self {
            Match::Container { visible_bounds, .. }
            | Match::Focusable { visible_bounds, .. }
            | Match::Scrollable { visible_bounds, .. }
            | Match::TextInput { visible_bounds, .. }
            | Match::Text { visible_bounds, .. }
            | Match::Custom { visible_bounds, .. } => *visible_bounds,
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

impl Bounded for Text {
    fn bounds(&self) -> Rectangle {
        match self {
            Text::Raw { bounds, .. } | Text::Input { bounds, .. } => *bounds,
        }
    }

    fn visible_bounds(&self) -> Option<Rectangle> {
        match self {
            Text::Raw { visible_bounds, .. }
            | Text::Input { visible_bounds, .. } => *visible_bounds,
        }
    }
}
