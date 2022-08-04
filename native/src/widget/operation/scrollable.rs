use crate::widget::{Id, Operation};

pub trait Scrollable {
    fn snap_to(&mut self, percentage: f32);
}

pub fn snap_to<T>(target: Id, percentage: f32) -> impl Operation<T> {
    struct SnapTo {
        target: Id,
        percentage: f32,
    }

    impl<T> Operation<T> for SnapTo {
        fn scrollable(&mut self, state: &mut dyn Scrollable, id: Option<&Id>) {
            if Some(&self.target) == id {
                state.snap_to(self.percentage);
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

    SnapTo { target, percentage }
}
