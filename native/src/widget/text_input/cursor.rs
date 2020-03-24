//! Track the cursor of a text input.
use crate::widget::text_input::Value;

#[derive(Debug, Copy, Clone)]
pub enum State {
    Index(usize),
    Selection { start: usize, end: usize },
}

#[derive(Debug, Copy, Clone)]
pub struct Cursor {
    state: State,
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor {
            state: State::Index(0),
        }
    }
}

impl Cursor {
    pub fn move_to(&mut self, position: usize) {
        self.state = State::Index(position);
    }

    pub fn move_right(&mut self, value: &Value) {
        self.move_right_by_amount(value, 1)
    }

    pub fn move_right_by_words(&mut self, value: &Value) {
        self.move_to(value.next_end_of_word(self.right(value)))
    }

    pub fn move_right_by_amount(&mut self, value: &Value, amount: usize) {
        match self.state(value) {
            State::Index(index) => {
                self.move_to(index.saturating_add(amount).min(value.len()))
            }
            State::Selection { start, end } => self.move_to(end.max(start)),
        }
    }

    pub fn move_left(&mut self, value: &Value) {
        match self.state(value) {
            State::Index(index) if index > 0 => self.move_to(index - 1),
            State::Selection { start, end } => self.move_to(start.min(end)),
            _ => self.move_to(0),
        }
    }

    pub fn move_left_by_words(&mut self, value: &Value) {
        self.move_to(value.previous_start_of_word(self.left(value)));
    }

    pub fn select_range(&mut self, start: usize, end: usize) {
        if start == end {
            self.state = State::Index(start);
        } else {
            self.state = State::Selection { start, end };
        }
    }

    pub fn select_left(&mut self, value: &Value) {
        match self.state(value) {
            State::Index(index) if index > 0 => {
                self.select_range(index, index - 1)
            }
            State::Selection { start, end } if end > 0 => {
                self.select_range(start, end - 1)
            }
            _ => (),
        }
    }

    pub fn select_right(&mut self, value: &Value) {
        match self.state(value) {
            State::Index(index) if index < value.len() => {
                self.select_range(index, index + 1)
            }
            State::Selection { start, end } if end < value.len() => {
                self.select_range(start, end + 1)
            }
            _ => (),
        }
    }

    pub fn select_left_by_words(&mut self, value: &Value) {
        match self.state(value) {
            State::Index(index) => {
                self.select_range(index, value.previous_start_of_word(index))
            }
            State::Selection { start, end } => {
                self.select_range(start, value.previous_start_of_word(end))
            }
        }
    }

    pub fn select_right_by_words(&mut self, value: &Value) {
        match self.state(value) {
            State::Index(index) => {
                self.select_range(index, value.next_end_of_word(index))
            }
            State::Selection { start, end } => {
                self.select_range(start, value.next_end_of_word(end))
            }
        }
    }

    pub fn select_all(&mut self, value: &Value) {
        self.select_range(0, value.len());
    }

    pub fn state(&self, value: &Value) -> State {
        match self.state {
            State::Index(index) => State::Index(index.min(value.len())),
            State::Selection { start, end } => {
                let start = start.min(value.len());
                let end = end.min(value.len());

                if start == end {
                    State::Index(start)
                } else {
                    State::Selection { start, end }
                }
            }
        }
    }

    pub fn start(&self, value: &Value) -> usize {
        let start = match self.state {
            State::Index(index) => index,
            State::Selection { start, .. } => start,
        };

        start.min(value.len())
    }

    pub fn end(&self, value: &Value) -> usize {
        let end = match self.state {
            State::Index(index) => index,
            State::Selection { end, .. } => end,
        };

        end.min(value.len())
    }

    fn left(&self, value: &Value) -> usize {
        match self.state(value) {
            State::Index(index) => index,
            State::Selection { start, end } => start.min(end),
        }
    }

    fn right(&self, value: &Value) -> usize {
        match self.state(value) {
            State::Index(index) => index,
            State::Selection { start, end } => start.max(end),
        }
    }

    pub fn selection(&self) -> Option<(usize, usize)> {
        match self.state {
            State::Selection { start, end } => {
                Some((start.min(end), start.max(end)))
            }
            _ => None,
        }
    }
}
