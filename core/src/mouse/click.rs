//! Track mouse clicks.
use crate::time::Instant;
use crate::Point;

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
    fn next(self) -> Kind {
        match self {
            Kind::Single | Kind::Triple => Kind::Double,
            Kind::Double => Kind::Triple,
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
        let duration = if time > self.time {
            Some(time - self.time)
        } else {
            None
        };

        self.position == new_position
            && duration.is_some_and(|duration| duration.as_millis() <= 300)
    }
}
