//! Operate on widgets that can be focused.
use crate::widget::operation::{Operation, Outcome, Rectangle};
use crate::widget::Id;

/// Produces an [`Operation`] that returns a layout of the widget
/// with the matching ID.
pub fn layout(target: Id) -> impl Operation<Rectangle> {
    struct Layout {
        target: Id,
        layout: Option<Rectangle>,
    }

    impl Operation<Rectangle> for Layout {
        fn layout(&mut self, layout: Rectangle, id: Option<&Id>) {
            if id.is_some() && id.unwrap() == &self.target {
                self.layout = Some(layout);
            }
        }

        fn container(
            &mut self,
            _id: Option<&Id>,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<Rectangle>),
        ) {
            operate_on_children(self)
        }

        fn finish(&self) -> Outcome<Rectangle> {
            if let Some(layout) = self.layout {
                Outcome::Some(layout)
            } else {
                Outcome::None
            }
        }
    }

    Layout {
        target,
        layout: None,
    }
}
