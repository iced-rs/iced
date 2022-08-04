use crate::widget::operation::{Operation, Outcome};
use crate::widget::Id;

pub trait Focusable {
    fn is_focused(&self) -> bool;
    fn focus(&mut self);
    fn unfocus(&mut self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Count {
    focused: Option<usize>,
    total: usize,
}

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
