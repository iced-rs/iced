//! Operate on widgets that can be focused.
use crate::widget::operation::{Operation, Outcome};
use crate::widget::Id;

/// The internal state of a widget that can be focused.
pub trait Focusable {
    /// Returns whether the widget is focused or not.
    fn is_focused(&self) -> bool;

    /// Focuses the widget.
    fn focus(&mut self);

    /// Unfocuses the widget.
    fn unfocus(&mut self);
}

/// A summary of the focusable widgets present on a widget tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Count {
    /// The index of the current focused widget, if any.
    focused: Option<usize>,

    /// The total amount of focusable widgets.
    total: usize,
}

/// Produces an [`Operation`] that focuses the widget with the given [`Id`].
pub fn focus<T>(target: Id) -> impl Operation<T> {
    struct Focus {
        target: Id,
    }

    impl<T> Operation<T> for Focus {
        fn focusable(&mut self, state: &mut dyn Focusable, id: Option<&Id>) {
            match id {
                Some(id) if id == &self.target => {
                    state.focus();
                }
                _ => {
                    state.unfocus();
                }
            }
        }

        fn container(
            &mut self,
            _id: Option<&Id>,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<T>),
        ) {
            operate_on_children(self)
        }
    }

    Focus { target }
}

/// Produces an [`Operation`] that generates a [`Count`] and chains it with the
/// provided function to build a new [`Operation`].
pub fn count<T, O>(f: fn(Count) -> O) -> impl Operation<T>
where
    O: Operation<T> + 'static,
{
    struct CountFocusable<O> {
        count: Count,
        next: fn(Count) -> O,
    }

    impl<T, O> Operation<T> for CountFocusable<O>
    where
        O: Operation<T> + 'static,
    {
        fn focusable(&mut self, state: &mut dyn Focusable, _id: Option<&Id>) {
            if state.is_focused() {
                self.count.focused = Some(self.count.total);
            }

            self.count.total += 1;
        }

        fn container(
            &mut self,
            _id: Option<&Id>,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<T>),
        ) {
            operate_on_children(self)
        }

        fn finish(&self) -> Outcome<T> {
            Outcome::Chain(Box::new((self.next)(self.count)))
        }
    }

    CountFocusable {
        count: Count::default(),
        next: f,
    }
}

/// Produces an [`Operation`] that searches for the current focused widget, and
/// - if found, focuses the previous focusable widget.
/// - if not found, focuses the last focusable widget.
pub fn focus_previous<T>() -> impl Operation<T> {
    struct FocusPrevious {
        count: Count,
        current: usize,
    }

    impl<T> Operation<T> for FocusPrevious {
        fn focusable(&mut self, state: &mut dyn Focusable, _id: Option<&Id>) {
            if self.count.total == 0 {
                return;
            }

            match self.count.focused {
                None if self.current == self.count.total - 1 => state.focus(),
                Some(0) if self.current == 0 => state.unfocus(),
                Some(0) => {}
                Some(focused) if focused == self.current => state.unfocus(),
                Some(focused) if focused - 1 == self.current => state.focus(),
                _ => {}
            }

            self.current += 1;
        }

        fn container(
            &mut self,
            _id: Option<&Id>,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<T>),
        ) {
            operate_on_children(self)
        }
    }

    count(|count| FocusPrevious { count, current: 0 })
}

/// Produces an [`Operation`] that searches for the current focused widget, and
/// - if found, focuses the next focusable widget.
/// - if not found, focuses the first focusable widget.
pub fn focus_next<T>() -> impl Operation<T> {
    struct FocusNext {
        count: Count,
        current: usize,
    }

    impl<T> Operation<T> for FocusNext {
        fn focusable(&mut self, state: &mut dyn Focusable, _id: Option<&Id>) {
            match self.count.focused {
                None if self.current == 0 => state.focus(),
                Some(focused) if focused == self.current => state.unfocus(),
                Some(focused) if focused + 1 == self.current => state.focus(),
                _ => {}
            }

            self.current += 1;
        }

        fn container(
            &mut self,
            _id: Option<&Id>,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<T>),
        ) {
            operate_on_children(self)
        }
    }

    count(|count| FocusNext { count, current: 0 })
}

/// Produces an [`Operation`] that searches for the current focused widget
/// and stores its ID. This ignores widgets that do not have an ID.
pub fn find_focused() -> impl Operation<Id> {
    struct FindFocused {
        focused: Option<Id>,
    }

    impl Operation<Id> for FindFocused {
        fn focusable(&mut self, state: &mut dyn Focusable, id: Option<&Id>) {
            if state.is_focused() && id.is_some() {
                self.focused = id.cloned();
            }
        }

        fn container(
            &mut self,
            _id: Option<&Id>,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<Id>),
        ) {
            operate_on_children(self)
        }

        fn finish(&self) -> Outcome<Id> {
            if let Some(id) = &self.focused {
                Outcome::Some(id.clone())
            } else {
                Outcome::None
            }
        }
    }

    FindFocused { focused: None }
}
