use crate::widget::text_input::Value;

#[derive(Debug, Copy, Clone)]
enum State {
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
    /* index move methods */
    pub fn move_to(&mut self, position: usize) {
        self.state = State::Index(position);
    }

    pub fn move_right(&mut self, value: &Value) {
        self.move_right_by_amount(value, 1)
    }

    pub fn move_right_by_words(&mut self, value: &Value) {
        self.move_to(value.next_end_of_word(self.right()))
    }

    pub fn move_right_by_amount(&mut self, value: &Value, amount: usize) {
        match self.state {
            State::Index(index) => {
                self.move_to(index.saturating_add(amount).min(value.len()))
            }
            State::Selection { .. } => self.move_to(self.right()),
        }
    }

    pub fn move_left(&mut self) {
        match self.state {
            State::Index(index) if index > 0 => self.move_to(index - 1),
            State::Selection { .. } => self.move_to(self.left()),
            _ => self.move_to(0),
        }
    }

    pub fn move_left_by_words(&mut self, value: &Value) {
        self.move_to(value.previous_start_of_word(self.right()));
    }
    /* end of index move methods */

    /* expand/shrink selection */
    // TODO: (whole section): Return State::Cursor if start == end after operation
    pub fn select_range(&mut self, start: usize, end: usize) {
        if start != end {
            self.state = State::Selection { start, end };
        } else {
            self.state = State::Index(start);
        }
    }

    pub fn select_left(&mut self) {
        match self.state {
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
        match self.state {
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
        match self.state {
            State::Index(index) => {
                self.select_range(index, value.previous_start_of_word(index))
            }
            State::Selection { start, end } => {
                self.select_range(start, value.previous_start_of_word(end))
            }
        }
    }

    pub fn select_right_by_words(&mut self, value: &Value) {
        match self.state {
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
    /* end of selection section */

    /* helpers */
    // get start position of selection (can be left OR right boundary of selection) or index
    pub(crate) fn start(&self) -> usize {
        match self.state {
            State::Index(index) => index,
            State::Selection { start, .. } => start,
        }
    }

    // get end position of selection (can be left OR right boundary of selection) or index
    pub fn end(&self) -> usize {
        match self.state {
            State::Index(index) => index,
            State::Selection { end, .. } => end,
        }
    }

    // get left boundary of selection or index
    pub fn left(&self) -> usize {
        match self.state {
            State::Index(index) => index,
            State::Selection { start, end } => start.min(end),
        }
    }

    // get right boundary of selection or index
    pub fn right(&self) -> usize {
        match self.state {
            State::Index(index) => index,
            State::Selection { start, end } => start.max(end),
        }
    }

    pub fn draw_position(&self, value: &Value) -> usize {
        self.cursor_position(value)
    }

    pub fn cursor_position(&self, value: &Value) -> usize {
        match self.state {
            State::Index(index) => index.min(value.len()),
            State::Selection { end, .. } => end.min(value.len()),
        }
    }

    // returns Option of left and right border of selection
    // a second method return start and end may be useful (see below)
    pub fn selection_position(&self) -> Option<(usize, usize)> {
        match self.state {
            State::Selection { start, end } => {
                Some((start.min(end), start.max(end)))
            }
            _ => None,
        }
    }

    /* pub fn selection_position(&self) -> Option<(usize, usize)> {
        match self.state {
            State::Selection { start, end } => Some((start, end)),
            _ => None,
        }
    } */
}
