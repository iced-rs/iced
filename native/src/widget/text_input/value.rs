use unicode_segmentation::UnicodeSegmentation;

use smol_str::SmolStr;
use std::fmt::{self, Display, Formatter};

/// The value of a [`TextInput`].
///
/// [`TextInput`]: crate::widget::TextInput
// TODO: Reduce allocations, cache results (?)
#[derive(Debug, Clone)]
pub struct Value {
    graphemes: Vec<SmolStr>,
}

impl Value {
    /// Creates a new [`Value`] from a string slice.
    pub fn new(string: &str) -> Self {
        let graphemes = UnicodeSegmentation::graphemes(string, true)
            .map(SmolStr::from)
            .collect();

        Self { graphemes }
    }

    /// Returns whether the [`Value`] is empty or not.
    ///
    /// A [`Value`] is empty when it contains no graphemes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the total amount of graphemes in the [`Value`].
    pub fn len(&self) -> usize {
        self.graphemes.len()
    }

    /// Returns the position of the previous start of a word from the given
    /// grapheme `index`.
    pub fn previous_start_of_word(&self, index: usize) -> usize {
        let previous_string =
            &self.graphemes[..index.min(self.graphemes.len())].concat();

        UnicodeSegmentation::split_word_bound_indices(previous_string.as_str())
            .rev()
            .find(|(_, word)| !word.trim_start().is_empty())
            .map(|(i, previous_word)| {
                index
                    - UnicodeSegmentation::graphemes(previous_word, true)
                        .count()
                    - UnicodeSegmentation::graphemes(
                        &previous_string[i + previous_word.len()..] as &str,
                        true,
                    )
                    .count()
            })
            .unwrap_or(0)
    }

    /// Returns the position of the next end of a word from the given grapheme
    /// `index`.
    pub fn next_end_of_word(&self, index: usize) -> usize {
        let next_string = &self.graphemes[index..].concat();

        #[allow(clippy::or_fun_call)]
        // self.len() is very cheap, we don't need lazy eval
        UnicodeSegmentation::split_word_bound_indices(next_string.as_str())
            .find(|(_, word)| !word.trim_start().is_empty())
            .map(|(i, next_word)| {
                index
                    + UnicodeSegmentation::graphemes(next_word, true).count()
                    + UnicodeSegmentation::graphemes(
                        &next_string[..i] as &str,
                        true,
                    )
                    .count()
            })
            .unwrap_or(self.len())
    }

    /// Returns an array containing the graphemes from `start` until the
    /// given `end`.
    pub fn select(&self, start: usize, end: usize) -> &[SmolStr] {
        &self.graphemes[start.min(self.len())..end.min(self.len())]
    }

    /// Returns an array containing the graphemes until the given `index`.
    pub fn until(&self, index: usize) -> &[SmolStr] {
        &self.graphemes[..index.min(self.len())]
    }

    /// Inserts a new `char` at the given grapheme `index`.
    pub fn insert(&mut self, index: usize, c: char) {
        let mut temp = [0_u8; 4];
        let tmp_str = c.encode_utf8(&mut temp);
        self.graphemes.insert(index, SmolStr::new_inline(tmp_str));

        self.graphemes = UnicodeSegmentation::graphemes(
            self.graphemes.concat().as_str(),
            true,
        )
        .map(SmolStr::from)
        .collect();
    }

    /// Inserts a bunch of graphemes at the given grapheme `index`.
    pub fn insert_many(&mut self, index: usize, mut value: Value) {
        let _ = self
            .graphemes
            .splice(index..index, value.graphemes.drain(..));
    }

    /// Removes the grapheme at the given `index`.
    pub fn remove(&mut self, index: usize) {
        let _ = self.graphemes.remove(index);
    }

    /// Removes the graphemes from `start` to `end`.
    pub fn remove_many(&mut self, start: usize, end: usize) {
        let _ = self.graphemes.splice(start..end, std::iter::empty());
    }

    /// Returns a new [`Value`] with all its graphemes replaced with the
    /// dot ('•') character.
    pub fn secure(&self) -> Self {
        Self {
            graphemes: std::iter::repeat(SmolStr::new_inline("•"))
                .take(self.graphemes.len())
                .collect(),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for grapheme in &self.graphemes {
            write!(f, "{}", grapheme)?;
        }
        Ok(())
    }
}
