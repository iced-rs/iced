//! Draw 2D graphics for your users.
pub mod event;

mod program;

pub use event::Event;
pub use program::Program;

pub use crate::graphics::cache::Group;
pub use crate::graphics::geometry::{
    fill, gradient, path, stroke, Fill, Gradient, LineCap, LineDash, LineJoin,
    Path, Stroke, Style, Text,
};

use crate::core;
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget::tree::{self, Tree};
use crate::core::{
    Clipboard, Element, Length, Rectangle, Shell, Size, Vector, Widget,
};
use crate::graphics::geometry;

use std::marker::PhantomData;

/// A simple cache that stores generated [`Geometry`] to avoid recomputation.
///
/// A [`Cache`] will not redraw its geometry unless the dimensions of its layer
/// change or it is explicitly cleared.
pub type Cache<Renderer = crate::Renderer> = geometry::Cache<Renderer>;

/// The geometry supported by a renderer.
pub type Geometry<Renderer = crate::Renderer> =
    <Renderer as geometry::Renderer>::Geometry;

/// The frame supported by a renderer.
pub type Frame<Renderer = crate::Renderer> = geometry::Frame<Renderer>;

/// A widget capable of drawing 2D graphics.
///
/// ## Drawing a simple circle
/// If you want to get a quick overview, here's how we can draw a simple circle:
///
/// ```no_run
/// # use iced_widget::canvas::{self, Canvas, Fill, Frame, Geometry, Path, Program};
/// # use iced_widget::core::{Color, Rectangle};
/// # use iced_widget::core::mouse;
/// # use iced_widget::{Renderer, Theme};
/// #
/// // First, we define the data we need for drawing
/// #[derive(Debug)]
/// struct Circle {
///     radius: f32,
/// }
///
/// // Then, we implement the `Program` trait
/// impl Program<()> for Circle {
///     type State = ();
///
///     fn draw(&self, _state: &(), renderer: &Renderer, _theme: &Theme, bounds: Rectangle, _cursor: mouse::Cursor) -> Vec<Geometry> {
///         // We prepare a new `Frame`
///         let mut frame = Frame::new(renderer, bounds.size());
///
///         // We create a `Path` representing a simple circle
///         let circle = Path::circle(frame.center(), self.radius);
///
///         // And fill it with some color
///         frame.fill(&circle, Color::BLACK);
///
///         // Finally, we produce the geometry
///         vec![frame.into_geometry()]
///     }
/// }
///
/// // Finally, we simply use our `Circle` to create the `Canvas`!
/// let canvas = Canvas::new(Circle { radius: 50.0 });
/// ```
#[derive(Debug)]
pub struct Canvas<P, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Renderer: geometry::Renderer,
    P: Program<Message, Theme, Renderer>,
{
    width: Length,
    height: Length,
    program: P,
    message_: PhantomData<Message>,
    theme_: PhantomData<Theme>,
    renderer_: PhantomData<Renderer>,
}

impl<P, Message, Theme, Renderer> Canvas<P, Message, Theme, Renderer>
where
    P: Program<Message, Theme, Renderer>,
    Renderer: geometry::Renderer,
{
    const DEFAULT_SIZE: f32 = 100.0;

    /// Creates a new [`Canvas`].
    pub fn new(program: P) -> Self {
        Canvas {
            width: Length::Fixed(Self::DEFAULT_SIZE),
            height: Length::Fixed(Self::DEFAULT_SIZE),
            program,
            message_: PhantomData,
            theme_: PhantomData,
            renderer_: PhantomData,
        }
    }

    /// Sets the width of the [`Canvas`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Canvas`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl<P, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Canvas<P, Message, Theme, Renderer>
where
    Renderer: geometry::Renderer,
    P: Program<Message, Theme, Renderer>,
{
    fn tag(&self) -> tree::Tag {
        struct Tag<T>(T);
        tree::Tag::of::<Tag<P::State>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(P::State::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: core::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let bounds = layout.bounds();

        let canvas_event = match event {
            core::Event::Mouse(mouse_event) => Some(Event::Mouse(mouse_event)),
            core::Event::Touch(touch_event) => Some(Event::Touch(touch_event)),
            core::Event::Keyboard(keyboard_event) => {
                Some(Event::Keyboard(keyboard_event))
            }
            core::Event::Window(_) => None,
        };

        if let Some(canvas_event) = canvas_event {
            let state = tree.state.downcast_mut::<P::State>();

            let (event_status, message) =
                self.program.update(state, canvas_event, bounds, cursor);

            if let Some(message) = message {
                shell.publish(message);
            }

            return event_status;
        }

        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<P::State>();

        self.program.mouse_interaction(state, bounds, cursor)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        if bounds.width < 1.0 || bounds.height < 1.0 {
            return;
        }

        let state = tree.state.downcast_ref::<P::State>();

        renderer.with_translation(
            Vector::new(bounds.x, bounds.y),
            |renderer| {
                let layers =
                    self.program.draw(state, renderer, theme, bounds, cursor);

                for layer in layers {
                    renderer.draw_geometry(layer);
                }
            },
        );
    }
}

impl<'a, P, Message, Theme, Renderer> From<Canvas<P, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: 'a + geometry::Renderer,
    P: 'a + Program<Message, Theme, Renderer>,
{
    fn from(
        canvas: Canvas<P, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(canvas)
    }
}
