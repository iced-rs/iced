///
///
#[derive(Debug, Default, Clone, Copy)]
pub struct IMERange {
    ///n bytes inserted before IME editing text.
    offset_bytes: usize,
    ///byte wise indexed.
    bold_line_range: Option<(usize, usize)>,
}

impl IMERange {
    pub fn offset_bytes(&self) -> usize {
        self.offset_bytes
    }
    pub fn set_offset_bytes(&mut self, offset_bytes: usize) {
        self.offset_bytes = offset_bytes;
    }
    pub fn set_range(&mut self, range: Option<(usize, usize)>) {
        self.bold_line_range = range
    }
    /// split text to three section of texts.

    /// * 1st section = light line text section.
    /// * 2nd section = bold line text section.
    /// * 3rd section = light line text section that after 2nd section.
    pub fn split_to_pieces(self, text: &str) -> [Option<&str>; 3] {
        match self.bold_line_range {
            Some((start, end)) => {
                if end == text.len() {
                    if start == 0 {
                        [None, Some(text), None]
                    } else {
                        let (first, second) = text.split_at(start);
                        [Some(first), Some(second), None]
                    }
                } else {
                    let third_section_offset = end - start;
                    let (first, second_third) = text.split_at(start);
                    let (second, third) =
                        second_third.split_at(third_section_offset);
                    [Some(first), Some(second), Some(third)]
                }
            }
            None => [None, Some(text), None],
        }
    }
    pub fn is_safe_to_split_text(self, text: &str) -> bool {
        if let Some((start, end)) = self.bold_line_range {
            (text.len() > start) && (text.len() > end)
        } else {
            true
        }
    }
}
