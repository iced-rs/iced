///
///
#[derive(Debug, Default, Clone, Copy)]
pub struct IMERange {
    ///n bytes inserted before IME editing text.
    offset_bytes: usize,
    candidate_indicator: Option<CandidateIndicator>,
}
#[derive(Debug, Clone, Copy)]
enum CandidateIndicator {
    // indicate like Windows
    BoldLine(usize, usize),
    // indicate like IBus-MOZC
    Cursor(usize),
}

impl IMERange {
    pub fn offset_bytes(&self) -> usize {
        self.offset_bytes
    }
    pub fn set_offset_bytes(&mut self, offset_bytes: usize) {
        self.offset_bytes = offset_bytes;
    }
    pub fn set_range(&mut self, range: Option<(usize, usize)>) {
        self.candidate_indicator = range.map(|(start, end)| {
            if start == end {
                CandidateIndicator::Cursor(start)
            } else {
                let left = start.min(end);
                let right = end.max(start);
                CandidateIndicator::BoldLine(left, right)
            }
        })
    }
    /// split text to three section of texts.

    /// * 1st section = light line text section.
    /// * 2nd section = bold line text section.
    /// * 3rd section = light line text section that after 2nd section.
    pub fn split_to_pieces(self, text: &str) -> [Option<&str>; 3] {
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
            None => [None, None, None],
        }
    }
    pub fn is_safe_to_split_text(self, text: &str) -> bool {
        if let Some(indicator) = self.candidate_indicator {
            match indicator {
                CandidateIndicator::BoldLine(start, end) => {
                    (text.len() > start) && (text.len() > end)
                }
                CandidateIndicator::Cursor(postition) => text.len() > postition,
            }
        } else {
            true
        }
    }
    pub fn before_cursor_text(self, text: &str) -> Option<&str> {
        if let Some(indicator) = self.candidate_indicator {
            match indicator {
                CandidateIndicator::BoldLine(_, _) => Some(text),
                CandidateIndicator::Cursor(position) => {
                    let (a, b) = text.split_at(position);
                    if a == "" {
                        Some(b)
                    } else {
                        Some(a)
                    }
                }
            }
        } else {
            Some(text)
        }
    }
}
