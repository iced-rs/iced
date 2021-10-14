//! Navigate an endless amount of content with a scrollbar.
use crate::{Backend, Renderer};
use iced_native::scrollable;
use iced_native::Rectangle;

pub use iced_native::scrollable::State;
pub use iced_style::scrollable::{Scrollbar, Scroller, StyleSheet};

/// A widget that can vertically display an infinite amount of content
/// with a scrollbar.
///
/// This is an alias of an `iced_native` scrollable with a default
/// `Renderer`.
pub type Scrollable<'a, Message, Backend> =
    iced_native::Scrollable<'a, Message, Renderer<Backend>>;

impl<B> scrollable::Renderer for Renderer<B>
where
    B: Backend,
{
    type Style = Box<dyn iced_style::scrollable::StyleSheet>;

    fn scrollbar(
        &self,
        bounds: Rectangle,
        content_bounds: Rectangle,
        offset: u32,
        scrollbar_width: u16,
        scrollbar_margin: u16,
        scroller_width: u16,
    ) -> Option<scrollable::Scrollbar> {
        if content_bounds.height > bounds.height {
            let outer_width =
                scrollbar_width.max(scroller_width) + 2 * scrollbar_margin;

            let outer_bounds = Rectangle {
                x: bounds.x + bounds.width - outer_width as f32,
                y: bounds.y,
                width: outer_width as f32,
                height: bounds.height,
            };

            let scrollbar_bounds = Rectangle {
                x: bounds.x + bounds.width
                    - f32::from(outer_width / 2 + scrollbar_width / 2),
                y: bounds.y,
                width: scrollbar_width as f32,
                height: bounds.height,
            };

            let ratio = bounds.height / content_bounds.height;
            let scroller_height = bounds.height * ratio;
            let y_offset = offset as f32 * ratio;

            let scroller_bounds = Rectangle {
                x: bounds.x + bounds.width
                    - f32::from(outer_width / 2 + scroller_width / 2),
                y: scrollbar_bounds.y + y_offset,
                width: scroller_width as f32,
                height: scroller_height,
            };

            Some(scrollable::Scrollbar {
                outer_bounds,
                bounds: scrollbar_bounds,
                margin: scrollbar_margin,
                scroller: scrollable::Scroller {
                    bounds: scroller_bounds,
                },
            })
        } else {
            None
        }
    }
}
