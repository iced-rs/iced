//! Operate on widgets that can have a text selection.
use crate::widget::{Id, Operation};
use crate::{Point, Rectangle};

/// The internal state of a widget that owns a text selection.
///
/// Coordinator widgets like `selectable_group` reach into selectables
/// through this trait via the [`Operation::selectable`] hook, the same
/// way [`Focusable`] / [`Operation::focusable`] cooperate.
///
/// New widgets only need to implement the required accessors:
/// selection state, the text content, the paragraph proxies, and the
/// "externally managed" flag. Codepoint / word / line walking is
/// provided.
///
/// [`Focusable`]: super::Focusable
/// [`Operation::focusable`]: Operation::focusable
/// [`Operation::selectable`]: Operation::selectable
pub trait Selectable {
    /// Returns the current selection as a half-open byte range, or
    /// `None` when nothing is selected.
    fn selection(&self) -> Option<(usize, usize)>;

    /// Sets the selection range. Pass `None` to clear.
    fn set_selection(&mut self, range: Option<(usize, usize)>);

    /// Returns the widget's text content. Used by the default
    /// implementations of [`text_len`], [`selection_text`],
    /// [`step_byte`], and [`step_byte_word`].
    ///
    /// [`text_len`]: Self::text_len
    /// [`selection_text`]: Self::selection_text
    /// [`step_byte`]: Self::step_byte
    /// [`step_byte_word`]: Self::step_byte_word
    fn text(&self) -> &str;

    /// Returns the visual position of `byte` in widget-local
    /// coordinates.
    fn byte_position(&self, byte: usize) -> Option<Point>;

    /// Hit-tests a widget-local point and returns the byte at that
    /// position.
    fn hit_test(&self, point: Point) -> Option<usize>;

    /// Returns the visual line height the widget renders with.
    fn visual_line_height(&self) -> Option<f32>;

    /// Returns the rendered text height. Default vertical stepping
    /// uses this to bail out past the last line so a coordinator can
    /// cross into a sibling.
    fn min_bounds_height(&self) -> f32;

    /// Returns the layout-bounds width. Default line-edge stepping
    /// clamps `Shift+End` against this.
    fn bounds_width(&self) -> f32;

    /// Marks the widget as externally managed. While `true`, the
    /// widget's own event handlers should skip drag-select and
    /// `Ctrl+C`, leaving its selection for an external coordinator
    /// to fill in.
    fn set_externally_managed(&mut self, value: bool);

    /// Returns the total length, in bytes, of the widget's text.
    fn text_len(&self) -> usize {
        self.text().len()
    }

    /// Returns the substring covered by the byte range
    /// `[start, end)`. Snaps to UTF-8 boundaries.
    fn selection_text(&self, start: usize, end: usize) -> String {
        let text = self.text();
        let start = floor_char_boundary(text, start);
        let end = floor_char_boundary(text, end);
        text.get(start..end).unwrap_or("").to_string()
    }

    /// Steps `byte` to the next or previous UTF-8 character
    /// boundary. `dir > 0` moves forward; `dir < 0` moves backward.
    fn step_byte(&self, byte: usize, dir: i32) -> usize {
        let text = self.text();
        let len = text.len();

        if dir > 0 {
            let mut next = (byte + 1).min(len);
            while next < len && !text.is_char_boundary(next) {
                next += 1;
            }
            next
        } else if byte == 0 {
            0
        } else {
            let mut prev = byte - 1;
            while prev > 0 && !text.is_char_boundary(prev) {
                prev -= 1;
            }
            prev
        }
    }

    /// Steps `byte` to the end of the next word (forward) or to the
    /// start of the previous word (backward), matching
    /// `text_input::next_end_of_word` / `previous_start_of_word`.
    fn step_byte_word(&self, byte: usize, dir: i32) -> usize {
        use unicode_segmentation::UnicodeSegmentation;

        let text = self.text();
        let len = text.len();

        if dir > 0 {
            let suffix = &text[byte..];
            suffix
                .split_word_bound_indices()
                .find(|(_, w)| !w.trim_start().is_empty())
                .map(|(i, w)| byte + i + w.len())
                .unwrap_or(len)
        } else {
            let prefix = &text[..byte];
            prefix
                .split_word_bound_indices()
                .rfind(|(_, w)| !w.trim_start().is_empty())
                .map(|(i, _)| i)
                .unwrap_or(0)
        }
    }

    /// Steps `byte` up or down one visual line. Returns `None` when
    /// the target falls outside the widget's rendered area.
    fn step_byte_line(&self, byte: usize, dir: i32) -> Option<usize> {
        let position = self.byte_position(byte)?;
        let line_height = self.visual_line_height()?;
        let target = Point::new(position.x, position.y + dir as f32 * line_height);

        if target.y < 0.0 || target.y >= self.min_bounds_height() {
            return None;
        }

        self.hit_test(target)
    }

    /// Returns the byte at the start (`dir < 0`) or end (`dir > 0`)
    /// of the visual line containing `byte`.
    fn line_edge_byte(&self, byte: usize, dir: i32) -> Option<usize> {
        let position = self.byte_position(byte)?;
        let target_x = if dir < 0 { 0.0 } else { self.bounds_width() };

        self.hit_test(Point::new(target_x, position.y))
    }
}

fn floor_char_boundary(s: &str, mut idx: usize) -> usize {
    if idx >= s.len() {
        return s.len();
    }

    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }

    idx
}

/// Produces an [`Operation`] that runs `callback` on every
/// [`Selectable`] in the operated subtree, in tree order.
pub fn visit<T, F>(callback: F) -> impl Operation<T>
where
    F: FnMut(Rectangle, &mut dyn Selectable) + Send,
{
    struct Visit<F> {
        callback: F,
    }

    impl<T, F> Operation<T> for Visit<F>
    where
        F: FnMut(Rectangle, &mut dyn Selectable) + Send,
    {
        fn selectable(
            &mut self,
            _id: Option<&Id>,
            bounds: Rectangle,
            state: &mut dyn Selectable,
        ) {
            (self.callback)(bounds, state);
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }
    }

    Visit { callback }
}
