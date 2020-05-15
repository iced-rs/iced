//! Zoom and pan on an image.
use crate::{
    image, layout, mouse, Clipboard, Element, Event, Hasher, Layout, Length,
    Point, Rectangle, Size, Widget,
};

use std::{f32, hash::Hash, u32};

/// A widget that can display an image with the ability to zoom in/out and pan.
#[allow(missing_debug_implementations)]
pub struct ImageViewer<'a> {
    state: &'a mut State,
    padding: u16,
    width: Length,
    height: Length,
    max_width: u32,
    max_height: u32,
    handle: image::Handle,
}

impl<'a> ImageViewer<'a> {
    /// Creates a new [`ImageViewer`] with the given [`State`] and [`Handle`].
    ///
    /// [`ImageViewer`]: struct.ImageViewer.html
    /// [`State`]: struct.State.html
    /// [`Handle`]: ../image/struct.Handle.html
    pub fn new(state: &'a mut State, handle: image::Handle) -> Self {
        ImageViewer {
            state,
            padding: 0,
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: u32::MAX,
            max_height: u32::MAX,
            handle,
        }
    }

    /// Sets the padding of the [`ImageViewer`].
    ///
    /// [`ImageViewer`]: struct.ImageViewer.html
    pub fn padding(mut self, units: u16) -> Self {
        self.padding = units;
        self
    }

    /// Sets the width of the [`ImageViewer`].
    ///
    /// [`ImageViewer`]: struct.ImageViewer.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`ImageViewer`].
    ///
    /// [`ImageViewer`]: struct.ImageViewer.html
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the max width of the [`ImageViewer`].
    ///
    /// [`ImageViewer`]: struct.ImageViewer.html
    pub fn max_width(mut self, max_width: u32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Sets the max height of the [`ImageViewer`].
    ///
    /// [`ImageViewer`]: struct.ImageViewer.html
    pub fn max_height(mut self, max_height: u32) -> Self {
        self.max_height = max_height;
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for ImageViewer<'a>
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
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let padding = f32::from(self.padding);

        let limits = limits
            .max_width(self.max_width)
            .max_height(self.max_height)
            .width(self.width)
            .height(self.height)
            .pad(padding);

        let size = limits.resolve(Size::INFINITY);

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
    ) {
        let bounds = layout.bounds();
        let is_mouse_over = bounds.contains(cursor_position);

        let image_bounds = {
            let (width, height) = renderer.dimensions(&self.handle);

            let dimensions = if let Some(scale) = self.state.scale {
                (width as f32 * scale, height as f32 * scale)
            } else {
                let dimensions = (width as f32, height as f32);

                let width_scale = bounds.width / dimensions.0;
                let height_scale = bounds.height / dimensions.1;

                let scale = width_scale.min(height_scale);

                if scale < 1.0 {
                    (dimensions.0 * scale, dimensions.1 * scale)
                } else {
                    (dimensions.0, dimensions.1)
                }
            };

            Rectangle {
                x: bounds.x,
                y: bounds.y,
                width: dimensions.0,
                height: dimensions.1,
            }
        };

        if is_mouse_over {
            match event {
                Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                    match delta {
                        mouse::ScrollDelta::Lines { y, .. }
                        | mouse::ScrollDelta::Pixels { y, .. } => {
                            // TODO: Configurable step and limits
                            if y > 0.0 {
                                self.state.scale = Some(
                                    (self.state.scale.unwrap_or(1.0) + 0.25)
                                        .min(10.0),
                                );
                            } else {
                                self.state.scale = Some(
                                    (self.state.scale.unwrap_or(1.0) - 0.25)
                                        .max(0.25),
                                );
                            }
                        }
                    }
                }
                Event::Mouse(mouse::Event::ButtonPressed(button)) => {
                    if button == mouse::Button::Left {
                        self.state.starting_cursor_pos =
                            Some((cursor_position.x, cursor_position.y));

                        self.state.starting_offset = self.state.current_offset;
                    }
                }
                Event::Mouse(mouse::Event::ButtonReleased(button)) => {
                    if button == mouse::Button::Left {
                        self.state.starting_cursor_pos = None
                    }
                }
                Event::Mouse(mouse::Event::CursorMoved { x, y }) => {
                    if self.state.is_cursor_clicked() {
                        self.state.pan(x, y, bounds, image_bounds);
                    }
                }
                _ => {}
            }
        } else if let Event::Mouse(mouse::Event::ButtonReleased(button)) = event
        {
            if button == mouse::Button::Left {
                self.state.starting_cursor_pos = None;
            }
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        let bounds = layout.bounds();

        let image_bounds = {
            let (width, height) = renderer.dimensions(&self.handle);

            let dimensions = if let Some(scale) = self.state.scale {
                (width as f32 * scale, height as f32 * scale)
            } else {
                let dimensions = (width as f32, height as f32);

                let width_scale = bounds.width / dimensions.0;
                let height_scale = bounds.height / dimensions.1;

                let scale = width_scale.min(height_scale);

                if scale < 1.0 {
                    (dimensions.0 * scale, dimensions.1 * scale)
                } else {
                    (dimensions.0, dimensions.1)
                }
            };

            Rectangle {
                x: bounds.x,
                y: bounds.y,
                width: dimensions.0,
                height: dimensions.1,
            }
        };

        let offset = self.state.offset(bounds, image_bounds);

        let is_mouse_over = bounds.contains(cursor_position);

        self::Renderer::draw(
            renderer,
            &self.state,
            bounds,
            image_bounds,
            offset,
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

/// The local state of an [`ImageViewer`].
///
/// [`ImageViewer`]: struct.ImageViewer.html
#[derive(Debug, Clone, Copy, Default)]
pub struct State {
    scale: Option<f32>,
    starting_offset: (f32, f32),
    current_offset: (f32, f32),
    starting_cursor_pos: Option<(f32, f32)>,
}

impl State {
    /// Creates a new [`State`].
    ///
    /// [`State`]: struct.State.html
    pub fn new() -> Self {
        State::default()
    }

    /// Apply a panning offset to the current [`State`], given the bounds of
    /// the [`ImageViewer`] and its image.
    ///
    /// [`ImageViewer`]: struct.ImageViewer.html
    /// [`State`]: struct.State.html
    fn pan(
        &mut self,
        x: f32,
        y: f32,
        bounds: Rectangle,
        image_bounds: Rectangle,
    ) {
        let delta_x = x - self.starting_cursor_pos.unwrap().0;
        let delta_y = y - self.starting_cursor_pos.unwrap().1;

        if bounds.width < image_bounds.width {
            self.current_offset.0 = (self.starting_offset.0 - delta_x)
                .max(0.0)
                .min((image_bounds.width - bounds.width) as f32);
        }

        if bounds.height < image_bounds.height {
            self.current_offset.1 = (self.starting_offset.1 - delta_y)
                .max(0.0)
                .min((image_bounds.height - bounds.height) as f32);
        }
    }

    /// Returns the current clipping offset of the [`State`], given the bounds
    /// of the [`ImageViewer`] and its contents.
    ///
    /// [`ImageViewer`]: struct.ImageViewer.html
    /// [`State`]: struct.State.html
    fn offset(&self, bounds: Rectangle, image_bounds: Rectangle) -> (u32, u32) {
        let hidden_width = ((image_bounds.width - bounds.width) as f32)
            .max(0.0)
            .round() as u32;

        let hidden_height = ((image_bounds.height - bounds.height) as f32)
            .max(0.0)
            .round() as u32;

        (
            (self.current_offset.0).min(hidden_width as f32) as u32,
            (self.current_offset.1).min(hidden_height as f32) as u32,
        )
    }

    /// Returns if the left mouse button is still held down since clicking inside
    /// the [`ImageViewer`].
    ///
    /// [`ImageViewer`]: struct.ImageViewer.html
    /// [`State`]: struct.State.html
    pub fn is_cursor_clicked(&self) -> bool {
        self.starting_cursor_pos.is_some()
    }
}

/// The renderer of an [`ImageViewer`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`ImageViewer`] in your user interface.
///
/// [`ImageViewer`]: struct.ImageViewer.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: crate::Renderer + Sized {
    /// Draws the [`ImageViewer`].
    ///
    /// It receives:
    /// - the [`State`] of the [`ImageViewer`]
    /// - the bounds of the [`ImageViewer`] widget
    /// - the bounds of the scaled [`ImageViewer`] image
    /// - the clipping x,y offset
    /// - the [`Handle`] to the underlying image
    /// - whether the mouse is over the [`ImageViewer`] or not
    ///
    /// [`ImageViewer`]: struct.ImageViewer.html
    /// [`State`]: struct.State.html
    /// [`Handle`]: ../image/struct.Handle.html
    fn draw(
        &mut self,
        state: &State,
        bounds: Rectangle,
        image_bounds: Rectangle,
        offset: (u32, u32),
        handle: image::Handle,
        is_mouse_over: bool,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<ImageViewer<'a>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer + image::Renderer,
    Message: 'a,
{
    fn from(viewer: ImageViewer<'a>) -> Element<'a, Message, Renderer> {
        Element::new(viewer)
    }
}
