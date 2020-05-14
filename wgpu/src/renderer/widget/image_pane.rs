use crate::{Primitive, Renderer};
use iced_native::{image, image_pane, mouse, Rectangle, Vector};

impl image_pane::Renderer for Renderer {
    fn draw(
        &mut self,
        state: &image_pane::State,
        bounds: Rectangle,
        image_bounds: Rectangle,
        offset: (u32, u32),
        handle: image::Handle,
        is_mouse_over: bool,
    ) -> Self::Output {
        (
            {
                Primitive::Clip {
                    bounds,
                    offset: Vector::new(offset.0, offset.1),
                    content: Box::new(Primitive::Image {
                        handle,
                        bounds: image_bounds,
                    }),
                }
            },
            {
                if state.is_cursor_clicked() {
                    mouse::Interaction::Grabbing
                } else if is_mouse_over
                    && (image_bounds.width > bounds.width
                        || image_bounds.height > bounds.height)
                {
                    mouse::Interaction::Grab
                } else {
                    mouse::Interaction::Idle
                }
            },
        )
    }
}
