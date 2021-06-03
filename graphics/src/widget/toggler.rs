//! Show toggle controls using togglers.
use crate::backend::{self, Backend};
use crate::{Primitive, Renderer};
use iced_native::mouse;
use iced_native::toggler;
use iced_native::Rectangle;

pub use iced_style::toggler::{Style, StyleSheet};

/// Makes sure that the border radius of the toggler looks good at every size.
const BORDER_RADIUS_RATIO: f32 = 32.0 / 13.0;

/// The space ratio between the background Quad and the Toggler bounds, and
/// between the background Quad and foreground Quad.
const SPACE_RATIO: f32 = 0.05;

/// A toggler that can be toggled.
///
/// This is an alias of an `iced_native` toggler with an `iced_wgpu::Renderer`.
pub type Toggler<Message, Backend> =
    iced_native::Toggler<Message, Renderer<Backend>>;

impl<B> toggler::Renderer for Renderer<B>
where
    B: Backend + backend::Text,
{
    type Style = Box<dyn StyleSheet>;

    const DEFAULT_SIZE: u16 = 20;

    fn draw(
        &mut self,
        bounds: Rectangle,
        is_active: bool,
        is_mouse_over: bool,
        label: Option<Self::Output>,
        style_sheet: &Self::Style,
    ) -> Self::Output {
        let style = if is_mouse_over {
            style_sheet.hovered(is_active)
        } else {
            style_sheet.active(is_active)
        };

        let border_radius = bounds.height as f32 / BORDER_RADIUS_RATIO;
        let space = SPACE_RATIO * bounds.height as f32;

        let toggler_background_bounds = Rectangle {
            x: bounds.x + space,
            y: bounds.y + space,
            width: bounds.width - (2.0 * space),
            height: bounds.height - (2.0 * space),
        };

        let toggler_background = Primitive::Quad {
            bounds: toggler_background_bounds,
            background: style.background.into(),
            border_radius,
            border_width: 1.0,
            border_color: style.background_border.unwrap_or(style.background),
        };

        let toggler_foreground_bounds = Rectangle {
            x: bounds.x
                + if is_active {
                    bounds.width - 2.0 * space - (bounds.height - (4.0 * space))
                } else {
                    2.0 * space
                },
            y: bounds.y + (2.0 * space),
            width: bounds.height - (4.0 * space),
            height: bounds.height - (4.0 * space),
        };

        let toggler_foreground = Primitive::Quad {
            bounds: toggler_foreground_bounds,
            background: style.foreground.into(),
            border_radius,
            border_width: 1.0,
            border_color: style.foreground_border.unwrap_or(style.foreground),
        };

        (
            Primitive::Group {
                primitives: match label {
                    Some((l, _)) => {
                        vec![l, toggler_background, toggler_foreground]
                    }
                    None => vec![toggler_background, toggler_foreground],
                },
            },
            if is_mouse_over {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::default()
            },
        )
    }
}
