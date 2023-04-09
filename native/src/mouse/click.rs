//! Track mouse clicks.
use crate::Point;
use std::time::Instant;

/// A mouse click.
#[derive(Debug, Clone, Copy)]
pub struct Click {
    kind: Kind,
    position: Point,
    time: Instant,
}

/// The kind of mouse click.
#[derive(Debug, Clone, Copy)]
pub enum Kind {
    /// A single click
    Single,

    /// A double click
    Double,

    /// A triple click
    Triple,
}

impl Kind {
    fn next(&self) -> Kind {
        match self {
            Kind::Single => Kind::Double,
            Kind::Double => Kind::Triple,
            Kind::Triple => Kind::Double,
        }
    }
}

impl Click {
    /// Creates a new [`Click`] with the given position and previous last
    /// [`Click`].
    pub fn new(position: Point, previous: Option<Click>) -> Click {
        let time = Instant::now();

        let kind = if let Some(previous) = previous {
            if previous.is_consecutive(position, time) {
                previous.kind.next()
            } else {
                Kind::Single
            }
        } else {
            Kind::Single
        };

        Click {
            kind,
            position,
            time,
        }
    }

    /// Returns the [`Kind`] of [`Click`].
    pub fn kind(&self) -> Kind {
        self.kind
    }

    fn is_consecutive(&self, new_position: Point, time: Instant) -> bool {
        self.position == new_position
            && time
                .checked_duration_since(self.time)
                .map(|duration| duration.as_millis() <= 300)
                .unwrap_or(false)
    }
}
