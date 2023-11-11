///
///
#[derive(Debug, Default, Clone)]
pub struct IMEState {
    preedit_text: String,
    candidate_indicator: Option<CandidateIndicator>,
}
#[derive(Debug, Clone, Copy)]
enum CandidateIndicator {
    // indicate like Windows
    BoldLine(usize, usize),
    // indicate like IBus-MOZC
    Cursor(usize),
}

impl IMEState {
    pub fn preedit_text(&self) -> &str {
        &self.preedit_text
    }
    pub fn set_event(
        &mut self,
        preedit_text: String,
        range: Option<(usize, usize)>,
    ) {
        self.preedit_text = preedit_text;
        // first we need to align to char boundary.
        //
        // ibus report incorrect charboundary for japanese input

        self.candidate_indicator = range.map(|(start, end)| {
            // utf-8 is 1 to 4 byte variable length encoding so we try +3 byte.
            let left = start.min(end);
            let right = end.clamp(start, self.preedit_text.len());
            let start_byte = (0..left + 1)
                .rfind(|index| self.preedit_text.is_char_boundary(*index));
            let end_byte = (right..right + 4)
                .find(|index| self.preedit_text.is_char_boundary(*index));
            if let Some((start, end)) = start_byte.zip(end_byte) {
                if start == end {
                    CandidateIndicator::Cursor(start)
                } else {
                    CandidateIndicator::BoldLine(start, end)
                }
            } else {
                CandidateIndicator::Cursor(self.preedit_text.len())
            }
        });
    }
    /// split text to three section of texts.

    /// * 1st section = light line text section.
    /// * 2nd section = bold line text section.
    /// * 3rd section = light line text section that after 2nd section.
    pub fn split_to_pieces(&self) -> [Option<&str>; 3] {
        let text = self.preedit_text.as_str();
        match self.candidate_indicator {
            Some(CandidateIndicator::BoldLine(start, end)) => {
                //split to three section.

                let (first, second_and_third) = if start < text.len() {
                    text.split_at(start)
                } else {
                    (text, "")
                };

                let (second, third) = if end < text.len() {
                    second_and_third.split_at(end - start)
                } else {
                    (second_and_third, "")
                };
                [Some(first), Some(second), Some(third)]
            }
            Some(CandidateIndicator::Cursor(_)) => [Some(text), None, None],
            None => [Some(text), None, None],
        }
    }

    pub fn before_cursor_text(&self) -> &str {
        let text = &self.preedit_text;
        if let Some(indicator) = self.candidate_indicator {
            match indicator {
                CandidateIndicator::BoldLine(_, _) => text,
                CandidateIndicator::Cursor(position) => {
                    let (a, b) = text.split_at(position);
                    if a.is_empty() & cfg!(windows) {
                        b
                    } else {
                        a
                    }
                }
            }
        } else {
            text
        }
    }
}
