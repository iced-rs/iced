pub mod focusable;
pub mod scrollable;

pub use focusable::Focusable;
pub use scrollable::Scrollable;

use crate::widget::Id;

pub trait Operation<T> {
    fn container(
        &mut self,
        id: Option<&Id>,
        operate_on_children: &mut dyn FnMut(&mut dyn Operation<T>),
    );

    fn focusable(&mut self, _state: &mut dyn Focusable, _id: Option<&Id>) {}

    fn scrollable(&mut self, _state: &mut dyn Scrollable, _id: Option<&Id>) {}

    fn finish(&self) -> Outcome<T> {
        Outcome::None
    }
}

pub enum Outcome<T> {
    None,
    Some(T),
    Chain(Box<dyn Operation<T>>),
}
