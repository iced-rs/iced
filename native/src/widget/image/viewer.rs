//! Zoom and pan on an image.
use crate::event::{self, Event};
use crate::image;
use crate::layout;
use crate::mouse;
use crate::{
    Clipboard, Element, Hasher, Layout, Length, Point, Rectangle, Size, Vector,
    Widget,
};

use std::{f32, hash::Hash, u32};

/// A frame that displays an image with the ability to zoom in/out and pan.
#[allow(missing_debug_implementations)]
pub struct Viewer<'a> {
    state: &'a mut State,
    padding: u16,
    width: Length,
    height: Length,
    max_width: u32,
    max_height: u32,
    min_scale: f32,
    max_scale: f32,
    scale_step: f32,
    handle: image::Handle,
}

impl<'a> Viewer<'a> {
    /// Creates a new [`Viewer`] with the given [`State`] and [`Handle`].
    ///
    /// [`Viewer`]: struct.Viewer.html
    /// [`State`]: struct.State.html
    /// [`Handle`]: ../../image/struct.Handle.html
    pub fn new(state: &'a mut State, handle: image::Handle) -> Self {
        Viewer {
            state,
            padding: 0,
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: u32::MAX,
            max_height: u32::MAX,
            min_scale: 0.25,
            max_scale: 10.0,
            scale_step: 0.10,
            handle,
        }
    }

    /// Sets the padding of the [`Viewer`].
    ///
    /// [`Viewer`]: struct.Viewer.html
    pub fn padding(mut self, units: u16) -> Self {
        self.padding = units;
        self
    }

    /// Sets the width of the [`Viewer`].
    ///
    /// [`Viewer`]: struct.Viewer.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Viewer`].
    ///
    /// [`Viewer`]: struct.Viewer.html
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the max width of the [`Viewer`].
    ///
    /// [`Viewer`]: struct.Viewer.html
    pub fn max_width(mut self, max_width: u32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Sets the max height of the [`Viewer`].
    ///
    /// [`Viewer`]: struct.Viewer.html
    pub fn max_height(mut self, max_height: u32) -> Self {
        self.max_height = max_height;
        self
    }

    /// Sets the max scale applied to the image of the [`Viewer`].
    ///
    /// Default is `10.0`
    ///
    /// [`Viewer`]: struct.Viewer.html
    pub fn max_scale(mut self, max_scale: f32) -> Self {
        self.max_scale = max_scale;
        self
    }

    /// Sets the min scale applied to the image of the [`Viewer`].
    ///
    /// Default is `0.25`
    ///
    /// [`Viewer`]: struct.Viewer.html
    pub fn min_scale(mut self, min_scale: f32) -> Self {
        self.min_scale = min_scale;
        self
    }

    /// Sets the percentage the image of the [`Viewer`] will be scaled by
    /// when zoomed in / out.
    ///
    /// Default is `0.10`
    ///
    /// [`Viewer`]: struct.Viewer.html
    pub fn scale_step(mut self, scale_step: f32) -> Self {
        self.scale_step = scale_step;
        self
    }

    /// Returns the bounds of the underlying image, given the bounds of
    /// the [`Viewer`]. Scaling will be applied and original aspect ratio
    /// will be respected.
    ///
    /// [`Viewer`]: struct.Viewer.html
    fn image_size<Renderer>(&self, renderer: &Renderer, bounds: Size) -> Size
    where
        Renderer: self::Renderer + image::Renderer,
    {
        let (width, height) = renderer.dimensions(&self.handle);

        let (width, height) = {
            let dimensions = (width as f32, height as f32);

            let width_ratio = bounds.width / dimensions.0;
            let height_ratio = bounds.height / dimensions.1;

            let ratio = width_ratio.min(height_ratio);

            let scale = self.state.scale;

            if ratio < 1.0 {
                (dimensions.0 * ratio * scale, dimensions.1 * ratio * scale)
            } else {
                (dimensions.0 * scale, dimensions.1 * scale)
            }
        };

        Size::new(width, height)
    }
}

/// Cursor position relative to the [`Viewer`] bounds.
///
/// [`Viewer`]: struct.Viewer.html
fn relative_cursor_position(
    absolute_position: Point,
    bounds: Rectangle,
) -> Point {
    absolute_position - Vector::new(bounds.x, bounds.y)
}

/// Center point relative to the [`Viewer`] bounds.
///
/// [`Viewer`]: struct.Viewer.html
fn relative_center(bounds: Rectangle) -> Point {
    bounds.center() - Vector::new(bounds.x, bounds.y)
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for Viewer<'a>
where
    Renderer: self::Renderer + image::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let (width, height) = renderer.dimensions(&self.handle);

        let aspect_ratio = width as f32 / height as f32;

        let mut size = limits
            .width(self.width)
            .height(self.height)
            .resolve(Size::new(width as f32, height as f32));

        let viewport_aspect_ratio = size.width / size.height;

        if viewport_aspect_ratio > aspect_ratio {
            size.width = width as f32 * size.height / height as f32;
        } else {
            size.height = height as f32 * size.width / width as f32;
        }

        layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _messages: &mut Vec<Message>,
        renderer: &Renderer,
        _clipboard: Option<&dyn Clipboard>,
    ) -> event::Status {
        let bounds = layout.bounds();
        let is_mouse_over = bounds.contains(cursor_position);

        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta })
                if is_mouse_over =>
            {
                match delta {
                    mouse::ScrollDelta::Lines { y, .. }
                    | mouse::ScrollDelta::Pixels { y, .. } => {
                        let previous_scale = self.state.scale;

                        if y < 0.0 && previous_scale > self.min_scale
                            || y > 0.0 && previous_scale < self.max_scale
                        {
                            self.state.scale = (if y > 0.0 {
                                self.state.scale * (1.0 + self.scale_step)
                            } else {
                                self.state.scale / (1.0 + self.scale_step)
                            })
                            .max(self.min_scale)
                            .min(self.max_scale);

                            let image_size =
                                self.image_size(renderer, bounds.size());

                            let factor =
                                self.state.scale / previous_scale - 1.0;

                            let cursor_to_center = relative_cursor_position(
                                cursor_position,
                                bounds,
                            ) - relative_center(bounds);

                            let adjustment = cursor_to_center * factor
                                + self.state.current_offset * factor;

                            self.state.current_offset = Vector::new(
                                if image_size.width > bounds.width {
                                    self.state.current_offset.x + adjustment.x
                                } else {
                                    0.0
                                },
                                if image_size.height > bounds.height {
                                    self.state.current_offset.y + adjustment.y
                                } else {
                                    0.0
                                },
                            );
                        }
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if is_mouse_over =>
            {
                self.state.cursor_grabbed_at = Some(cursor_position);
                self.state.starting_offset = self.state.current_offset;
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                self.state.cursor_grabbed_at = None
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if let Some(origin) = self.state.cursor_grabbed_at {
                    let image_size = self.image_size(renderer, bounds.size());

                    let hidden_width = (image_size.width - bounds.width / 2.0)
                        .max(0.0)
                        .round();

                    let hidden_height = (image_size.height
                        - bounds.height / 2.0)
                        .max(0.0)
                        .round();

                    let delta = position - origin;

                    let x = if bounds.width < image_size.width {
                        (self.state.starting_offset.x - delta.x)
                            .min(hidden_width)
                            .max(-hidden_width)
                    } else {
                        0.0
                    };

                    let y = if bounds.height < image_size.height {
                        (self.state.starting_offset.y - delta.y)
                            .min(hidden_height)
                            .max(-hidden_height)
                    } else {
                        0.0
                    };

                    self.state.current_offset = Vector::new(x, y);
                }
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> Renderer::Output {
        let bounds = layout.bounds();

        let image_size = self.image_size(renderer, bounds.size());

        let translation = {
            let image_top_left = Vector::new(
                bounds.width / 2.0 - image_size.width / 2.0,
                bounds.height / 2.0 - image_size.height / 2.0,
            );

            image_top_left - self.state.offset(bounds, image_size)
        };

        let is_mouse_over = bounds.contains(cursor_position);

        self::Renderer::draw(
            renderer,
            &self.state,
            bounds,
            image_size,
            translation,
            self.handle.clone(),
            is_mouse_over,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.height.hash(state);
        self.max_width.hash(state);
        self.max_height.hash(state);
        self.padding.hash(state);

        self.handle.hash(state);
    }
}

/// The local state of a [`Viewer`].
///
/// [`Viewer`]: struct.Viewer.html
#[derive(Debug, Clone, Copy)]
pub struct State {
    scale: f32,
    starting_offset: Vector,
    current_offset: Vector,
    cursor_grabbed_at: Option<Point>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            scale: 1.0,
            starting_offset: Vector::default(),
            current_offset: Vector::default(),
            cursor_grabbed_at: None,
        }
    }
}

impl State {
    /// Creates a new [`State`].
    ///
    /// [`State`]: struct.State.html
    pub fn new() -> Self {
        State::default()
    }

    /// Returns the current offset of the [`State`], given the bounds
    /// of the [`Viewer`] and its image.
    ///
    /// [`Viewer`]: struct.Viewer.html
    /// [`State`]: struct.State.html
    fn offset(&self, bounds: Rectangle, image_size: Size) -> Vector {
        let hidden_width =
            (image_size.width - bounds.width / 2.0).max(0.0).round();

        let hidden_height =
            (image_size.height - bounds.height / 2.0).max(0.0).round();

        Vector::new(
            self.current_offset.x.min(hidden_width).max(-hidden_width),
            self.current_offset.y.min(hidden_height).max(-hidden_height),
        )
    }

    /// Returns if the cursor is currently grabbed by the [`Viewer`].
    ///
    /// [`Viewer`]: struct.Viewer.html
    /// [`State`]: struct.State.html
    pub fn is_cursor_grabbed(&self) -> bool {
        self.cursor_grabbed_at.is_some()
    }
}

/// The renderer of an [`Viewer`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`Viewer`] in your user interface.
///
/// [`Viewer`]: struct.Viewer.html
/// [renderer]: ../../../renderer/index.html
pub trait Renderer: crate::Renderer + Sized {
    /// Draws the [`Viewer`].
    ///
    /// It receives:
    /// - the [`State`] of the [`Viewer`]
    /// - the bounds of the [`Viewer`] widget
    /// - the bounds of the scaled [`Viewer`] image
    /// - the translation of the clipped image
    /// - the [`Handle`] to the underlying image
    /// - whether the mouse is over the [`Viewer`] or not
    ///
    /// [`Viewer`]: struct.Viewer.html
    /// [`State`]: struct.State.html
    /// [`Handle`]: ../../image/struct.Handle.html
    fn draw(
        &mut self,
        state: &State,
        bounds: Rectangle,
        image_size: Size,
        translation: Vector,
        handle: image::Handle,
        is_mouse_over: bool,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<Viewer<'a>> for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer + image::Renderer,
    Message: 'a,
{
    fn from(viewer: Viewer<'a>) -> Element<'a, Message, Renderer> {
        Element::new(viewer)
    }
}
