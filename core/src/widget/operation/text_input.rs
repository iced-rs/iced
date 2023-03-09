//! Operate on widgets that have text input.
use crate::widget::operation::Operation;
use crate::widget::Id;

/// The internal state of a widget that has text input.
pub trait TextInput {
    /// Moves the cursor of the text input to the front of the input text.
    fn move_cursor_to_front(&mut self);
    /// Moves the cursor of the text input to the end of the input text.
    fn move_cursor_to_end(&mut self);
    /// Moves the cursor of the text input to an arbitrary location.
    fn move_cursor_to(&mut self, position: usize);
    /// Selects all the content of the text input.
    fn select_all(&mut self);
}

/// Produces an [`Operation`] that moves the cursor of the widget with the given [`Id`] to the
/// front.
pub fn move_cursor_to_front<T>(target: Id) -> impl Operation<T> {
    struct MoveCursor {
        target: Id,
    }

    impl<T> Operation<T> for MoveCursor {
        fn text_input(&mut self, state: &mut dyn TextInput, id: Option<&Id>) {
            match id {
                Some(id) if id == &self.target => {
                    state.move_cursor_to_front();
                }
                _ => {}
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

    MoveCursor { target }
}

/// Produces an [`Operation`] that moves the cursor of the widget with the given [`Id`] to the
/// end.
pub fn move_cursor_to_end<T>(target: Id) -> impl Operation<T> {
    struct MoveCursor {
        target: Id,
    }

    impl<T> Operation<T> for MoveCursor {
        fn text_input(&mut self, state: &mut dyn TextInput, id: Option<&Id>) {
            match id {
                Some(id) if id == &self.target => {
                    state.move_cursor_to_end();
                }
                _ => {}
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

    MoveCursor { target }
}

/// Produces an [`Operation`] that moves the cursor of the widget with the given [`Id`] to the
/// provided position.
pub fn move_cursor_to<T>(target: Id, position: usize) -> impl Operation<T> {
    struct MoveCursor {
        target: Id,
        position: usize,
    }

    impl<T> Operation<T> for MoveCursor {
        fn text_input(&mut self, state: &mut dyn TextInput, id: Option<&Id>) {
            match id {
                Some(id) if id == &self.target => {
                    state.move_cursor_to(self.position);
                }
                _ => {}
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

    MoveCursor { target, position }
}

/// Produces an [`Operation`] that selects all the content of the widget with the given [`Id`].
pub fn select_all<T>(target: Id) -> impl Operation<T> {
    struct MoveCursor {
        target: Id,
    }

    impl<T> Operation<T> for MoveCursor {
        fn text_input(&mut self, state: &mut dyn TextInput, id: Option<&Id>) {
            match id {
                Some(id) if id == &self.target => {
                    state.select_all();
                }
                _ => {}
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

    MoveCursor { target }
}
