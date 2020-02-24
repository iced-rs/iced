//! Build mouse events.
mod button;
mod event;

use crate::Point;
pub use button::Button;
pub use event::{Event, ScrollDelta};
use std::time::{Duration, SystemTime};

/// enum to track the type of the last click
#[derive(Debug, Copy, Clone)]
pub enum Interaction {
    /// Last Click was a single click
    Click(Point),
    /// Last Click was a double click
    DoubleClick(Point),
    /// Last Click was a triple click
    TripleClick(Point),
}

/// Compiler bully
#[derive(Debug, Copy, Clone)]
pub struct State {
    last_click: Option<Interaction>,
    last_click_timestamp: Option<SystemTime>,
}

impl Default for State {
    fn default() -> Self {
        State {
            last_click: None,
            last_click_timestamp: None,
        }
    }
}

impl State {
    /// processes left click to check for double/triple clicks
    /// return amount of repetitive mouse clicks
    /// (1 -> double click, 2 -> triple click)
    pub fn update(&mut self, position: Point) -> Interaction {
        self.last_click_timestamp = Some(SystemTime::now());
        self.last_click = match self.last_click {
            None => Some(Interaction::Click(position)),
            Some(x) => match x {
                Interaction::Click(p) if self.process_click(p, position) => {
                    Some(Interaction::DoubleClick(position))
                }
                Interaction::DoubleClick(p)
                    if self.process_click(p, position) =>
                {
                    Some(Interaction::TripleClick(position))
                }
                _ => Some(Interaction::Click(position)),
            },
        };
        self.last_click.unwrap_or(Interaction::Click(position))
    }

    fn process_click(&self, old_position: Point, new_position: Point) -> bool {
        old_position == new_position
            && SystemTime::now()
                .duration_since(
                    self.last_click_timestamp.unwrap_or(SystemTime::UNIX_EPOCH),
                )
                .unwrap_or(Duration::from_secs(1))
                .as_millis()
                <= 500
    }
}
