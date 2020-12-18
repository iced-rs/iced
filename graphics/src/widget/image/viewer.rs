//! Zoom and pan on an image.
use crate::backend::{self, Backend};
use crate::{Primitive, Renderer};

use iced_native::image;
use iced_native::image::viewer;
use iced_native::mouse;
use iced_native::{Rectangle, Size, Vector};

impl<B> viewer::Renderer for Renderer<B>
where
    B: Backend + backend::Image,
{
    fn draw(
        &mut self,
        state: &viewer::State,
        bounds: Rectangle,
        image_size: Size,
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
                            bounds: Rectangle {
                                x: bounds.x,
                                y: bounds.y,
                                ..Rectangle::with_size(image_size)
                            },
                        }),
                    }),
                    offset: Vector::new(0, 0),
                }
            },
            {
                if state.is_cursor_grabbed() {
                    mouse::Interaction::Grabbing
                } else if is_mouse_over
                    && (image_size.width > bounds.width
                        || image_size.height > bounds.height)
                {
                    mouse::Interaction::Grab
                } else {
                    mouse::Interaction::Idle
                }
            },
        )
    }
}
