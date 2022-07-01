//! Draw 2D graphics for your users.
//!
//! A [`Canvas`] widget can be used to draw different kinds of 2D shapes in a
//! [`Frame`]. It can be used for animation, data visualization, game graphics,
//! and more!
mod program;

pub use crate::widget::canvas::{Canvas as _, Program as _, *};

pub use program::Program;

use crate::{Backend, Primitive, Renderer};

use iced_native::layout::{self, Layout};
use iced_native::mouse;
use iced_native::renderer;
use iced_native::{Clipboard, Length, Point, Rectangle, Shell, Size, Vector};
use iced_pure::widget::tree::{self, Tree};
use iced_pure::{Element, Widget};

use std::marker::PhantomData;

/// A widget capable of drawing 2D graphics.
///
/// ## Drawing a simple circle
/// If you want to get a quick overview, here's how we can draw a simple circle:
///
/// ```no_run
/// # mod iced {
/// #     pub mod pure {
/// #         pub use iced_graphics::pure::canvas;
/// #     }
/// #     pub use iced_native::{Color, Rectangle};
/// # }
/// use iced::pure::canvas::{self, Canvas, Cursor, Fill, Frame, Geometry, Path, Program};
/// use iced::{Color, Rectangle};
///
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
///     fn draw(&self, _state: &(), bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry>{
///         // We prepare a new `Frame`
///         let mut frame = Frame::new(bounds.size());
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
pub struct Canvas<Message, P>
where
    P: Program<Message>,
{
    width: Length,
    height: Length,
    program: P,
    message_: PhantomData<Message>,
}

impl<Message, P> Canvas<Message, P>
where
    P: Program<Message>,
{
    const DEFAULT_SIZE: u16 = 100;

    /// Creates a new [`Canvas`].
    pub fn new(program: P) -> Self {
        Canvas {
            width: Length::Units(Self::DEFAULT_SIZE),
            height: Length::Units(Self::DEFAULT_SIZE),
            program,
            message_: PhantomData,
        }
    }

    /// Sets the width of the [`Canvas`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Canvas`].
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }
}

impl<Message, P, B> Widget<Message, Renderer<B>> for Canvas<Message, P>
where
    P: Program<Message>,
    B: Backend,
{
    fn tag(&self) -> tree::Tag {
        struct Tag<T>(T);
        tree::Tag::of::<Tag<P::State>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(P::State::default())
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        _renderer: &Renderer<B>,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);
        let size = limits.resolve(Size::ZERO);

        layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: iced_native::Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _renderer: &Renderer<B>,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let bounds = layout.bounds();

        let canvas_event = match event {
            iced_native::Event::Mouse(mouse_event) => {
                Some(Event::Mouse(mouse_event))
            }
            iced_native::Event::Keyboard(keyboard_event) => {
                Some(Event::Keyboard(keyboard_event))
            }
            _ => None,
        };

        let cursor = Cursor::from_window_position(cursor_position);

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
        cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer<B>,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
        let cursor = Cursor::from_window_position(cursor_position);
        let state = tree.state.downcast_ref::<P::State>();

        self.program.mouse_interaction(state, bounds, cursor)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer<B>,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        use iced_native::Renderer as _;

        let bounds = layout.bounds();

        if bounds.width < 1.0 || bounds.height < 1.0 {
            return;
        }

        let translation = Vector::new(bounds.x, bounds.y);
        let cursor = Cursor::from_window_position(cursor_position);
        let state = tree.state.downcast_ref::<P::State>();

        renderer.with_translation(translation, |renderer| {
            renderer.draw_primitive(Primitive::Group {
                primitives: self
                    .program
                    .draw(state, bounds, cursor)
                    .into_iter()
                    .map(Geometry::into_primitive)
                    .collect(),
            });
        });
    }
}

impl<'a, Message, P, B> From<Canvas<Message, P>>
    for Element<'a, Message, Renderer<B>>
where
    Message: 'a,
    P: Program<Message> + 'a,
    B: Backend,
{
    fn from(canvas: Canvas<Message, P>) -> Element<'a, Message, Renderer<B>> {
        Element::new(canvas)
    }
}
