//! Zoom and pan on an image.
use crate::backend::{self, Backend};
use crate::{Primitive, Renderer};

use iced_native::image;
use iced_native::image::viewer;
use iced_native::mouse;
use iced_native::{Rectangle, Vector};

impl<B> viewer::Renderer for Renderer<B>
where
    B: Backend + backend::Image,
{
    fn draw(
        &mut self,
        state: &viewer::State,
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
