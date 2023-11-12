use iced_renderer::core::text::Paragraph;

///
///
#[derive(Debug, Default, Clone)]
pub struct IMEState<P: Paragraph> {
    before_preedit_text: String,
    before_preedit_paragraph: P,
    preedit_text: String,
    whole_paragraph: P,
    underlines: Option<[(f32, f32); 3]>,
    candidate_indicator: Option<CandidateIndicator>,
}

#[derive(Debug, Clone, Copy)]
enum CandidateIndicator {
    // indicate like Windows
    BoldLine(usize, usize),
    // indicate like IBus-MOZC
    Cursor(usize),
}

impl<P: Paragraph> IMEState<P> {
    pub fn before_preedit_text(&self) -> &str {
        &self.before_preedit_text
    }
    pub fn set_before_preedit_text(&mut self, text: String) {
        self.before_preedit_text = text;
    }
    pub fn before_preedit_paragraph_mut(&mut self) -> &mut P {
        &mut self.before_preedit_paragraph
    }
    pub fn before_preedit_paragraph(&self) -> &P {
        &self.before_preedit_paragraph
    }
    pub fn whole_paragraph_mut(&mut self) -> &mut P {
        &mut self.whole_paragraph
    }
    pub fn whole_paragraph(&self) -> &P {
        &self.whole_paragraph
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

    /// measure underline offset and width for each splitted pieces.
    ///
    /// we can retrieve result by underlines method.
    pub fn measure_underlines<F: Fn(&str) -> f32>(&mut self, measure_fn: F) {
        let pieces = self.split_to_pieces();
        let mut width_iter = pieces.iter().map(|chunk| match chunk {
            Some(chunk) => (measure_fn)(chunk),
            None => 0.0,
        });

        let mut widths = [0.0; 3];
        widths.fill_with(|| width_iter.next().unwrap());
        let _ = self.underlines.replace([
            (0.0, widths[0]),
            (widths[0], widths[1]),
            (widths[0] + widths[1], widths[2]),
        ]);
    }
    /// retrieve underline infos
    ///
    pub fn underlines(&self) -> Option<[(f32, f32); 3]> {
        self.underlines
    }
}
