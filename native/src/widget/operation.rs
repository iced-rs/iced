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
