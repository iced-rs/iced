use crate::{Primitive, Renderer};
use iced_native::{image, mouse, Rectangle, Vector};

impl image::viewer::Renderer for Renderer {
    fn draw(
        &mut self,
        state: &image::State,
        bounds: Rectangle,
        image_bounds: Rectangle,
        translation: Vector,
        handle: image::Handle,
        is_mouse_over: bool,
    ) -> Self::Output {
        (
            {
                Primitive::Clip {
                    bounds,
                    content: Box::new(Primitive::Translate {
                        translation,
                        content: Box::new(Primitive::Image {
                            handle,
                            bounds: image_bounds,
                        }),
                    }),
                    offset: Vector::new(0, 0),
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
