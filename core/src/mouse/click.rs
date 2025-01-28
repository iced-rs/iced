//! Track mouse clicks.
use crate::mouse::Button;
use crate::time::Instant;
use crate::{Point, Transformation};

use std::ops::Mul;

/// A mouse click.
#[derive(Debug, Clone, Copy)]
pub struct Click {
    kind: Kind,
    button: Button,
    position: Point,
    time: Instant,
}

/// The kind of mouse click.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            Kind::Single => Kind::Double,
            Kind::Double => Kind::Triple,
            Kind::Triple => Kind::Double,
        }
    }
}

impl Click {
    /// Creates a new [`Click`] with the given position and previous last
    /// [`Click`].
    pub fn new(
        position: Point,
        button: Button,
        previous: Option<Click>,
    ) -> Click {
        let time = Instant::now();

        let kind = if let Some(previous) = previous {
            if previous.is_consecutive(position, time)
                && button == previous.button
            {
                previous.kind.next()
            } else {
                Kind::Single
            }
        } else {
            Kind::Single
        };

        Click {
            kind,
            button,
            position,
            time,
        }
    }

    /// Returns the [`Kind`] of [`Click`].
    pub fn kind(&self) -> Kind {
        self.kind
    }

    /// Returns the position of the [`Click`].
    pub fn position(&self) -> Point {
        self.position
    }

    fn is_consecutive(&self, new_position: Point, time: Instant) -> bool {
        let duration = if time > self.time {
            Some(time - self.time)
        } else {
            None
        };

        self.position.distance(new_position) < 6.0
            && duration
                .map(|duration| duration.as_millis() <= 300)
                .unwrap_or(false)
    }
}

impl Mul<Transformation> for Click {
    type Output = Click;

    fn mul(self, transformation: Transformation) -> Click {
        Click {
            kind: self.kind,
            button: self.button,
            position: self.position * transformation,
            time: self.time,
        }
    }
}
