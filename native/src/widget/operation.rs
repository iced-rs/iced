use crate::widget::state;
use crate::widget::Id;

pub trait Operation<T> {
    fn container(
        &mut self,
        id: Option<&Id>,
        operate_on_children: &mut dyn FnMut(&mut dyn Operation<T>),
    );

    fn focusable(
        &mut self,
        _state: &mut dyn state::Focusable,
        _id: Option<&Id>,
    ) {
    }

    fn finish(&self) -> Outcome<T> {
        Outcome::None
    }
}

pub enum Outcome<T> {
    None,
    Some(T),
    Chain(Box<dyn Operation<T>>),
}

pub fn focus<T>(target: Id) -> impl Operation<T> {
    struct Focus {
        target: Id,
    }

    impl<T> Operation<T> for Focus {
        fn focusable(
            &mut self,
            state: &mut dyn state::Focusable,
            id: Option<&Id>,
        ) {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FocusCount {
    focused: Option<usize>,
    total: usize,
}

pub fn count_focusable<T, O>(f: fn(FocusCount) -> O) -> impl Operation<T>
where
    O: Operation<T> + 'static,
{
    struct CountFocusable<O> {
        count: FocusCount,
        next: fn(FocusCount) -> O,
    }

    impl<T, O> Operation<T> for CountFocusable<O>
    where
        O: Operation<T> + 'static,
    {
        fn focusable(
            &mut self,
            state: &mut dyn state::Focusable,
            _id: Option<&Id>,
        ) {
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
        count: FocusCount::default(),
        next: f,
    }
}

pub fn focus_next<T>() -> impl Operation<T> {
    struct FocusNext {
        count: FocusCount,
        current: usize,
    }

    impl<T> Operation<T> for FocusNext {
        fn focusable(
            &mut self,
            state: &mut dyn state::Focusable,
            _id: Option<&Id>,
        ) {
            if self.count.total == 0 {
                return;
            }

            match self.count.focused {
                None if self.current == 0 => state.focus(),
                Some(focused) if focused == self.current => state.unfocus(),
                Some(focused) if focused + 1 == self.current => state.focus(),
                Some(focused)
                    if focused == self.count.total - 1 && self.current == 0 =>
                {
                    state.focus()
                }
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

    count_focusable(|count| FocusNext { count, current: 0 })
}
