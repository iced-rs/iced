//! A custom shader widget for wgpu applications.
mod event;
mod program;

pub use event::Event;
pub use program::Program;

use crate::core;
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget::tree::{self, Tree};
use crate::core::widget::{self, Widget};
use crate::core::window;
use crate::core::{Clipboard, Element, Length, Rectangle, Shell, Size};
use crate::renderer::wgpu::primitive;

use std::marker::PhantomData;

pub use crate::graphics::Viewport;
pub use crate::renderer::wgpu::wgpu;
pub use primitive::{Primitive, Storage};

/// A widget which can render custom shaders with Iced's `wgpu` backend.
///
/// Must be initialized with a [`Program`], which describes the internal widget state & how
/// its [`Program::Primitive`]s are drawn.
#[allow(missing_debug_implementations)]
pub struct Shader<Message, P: Program<Message>> {
    width: Length,
    height: Length,
    program: P,
    _message: PhantomData<Message>,
}

impl<Message, P: Program<Message>> Shader<Message, P> {
    /// Create a new custom [`Shader`].
    pub fn new(program: P) -> Self {
        Self {
            width: Length::Fixed(100.0),
            height: Length::Fixed(100.0),
            program,
            _message: PhantomData,
        }
    }

    /// Set the `width` of the custom [`Shader`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Set the `height` of the custom [`Shader`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl<P, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Shader<Message, P>
where
    P: Program<Message>,
    Renderer: primitive::Renderer,
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
        event: crate::core::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let bounds = layout.bounds();

        let custom_shader_event = match event {
            core::Event::Mouse(mouse_event) => Some(Event::Mouse(mouse_event)),
            core::Event::Keyboard(keyboard_event) => {
                Some(Event::Keyboard(keyboard_event))
            }
            core::Event::Touch(touch_event) => Some(Event::Touch(touch_event)),
            core::Event::Window(window::Event::RedrawRequested(instant)) => {
                Some(Event::RedrawRequested(instant))
            }
            core::Event::Window(_) => None,
        };

        if let Some(custom_shader_event) = custom_shader_event {
            let state = tree.state.downcast_mut::<P::State>();

            let (event_status, message) = self.program.update(
                state,
                custom_shader_event,
                bounds,
                cursor,
                shell,
            );

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
        tree: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<P::State>();

        renderer.draw_primitive(
            bounds,
            self.program.draw(state, cursor_position, bounds),
        );
    }
}

impl<'a, Message, Theme, Renderer, P> From<Shader<Message, P>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: primitive::Renderer,
    P: Program<Message> + 'a,
{
    fn from(
        custom: Shader<Message, P>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(custom)
    }
}

impl<Message, T> Program<Message> for &T
where
    T: Program<Message>,
{
    type State = T::State;
    type Primitive = T::Primitive;

    fn update(
        &self,
        state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) -> (event::Status, Option<Message>) {
        T::update(self, state, event, bounds, cursor, shell)
    }

    fn draw(
        &self,
        state: &Self::State,
        cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        T::draw(self, state, cursor, bounds)
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        T::mouse_interaction(self, state, bounds, cursor)
    }
}
