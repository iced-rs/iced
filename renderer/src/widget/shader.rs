//! A custom shader widget for wgpu applications.
use crate::core::event::Status;
use crate::core::layout::{Limits, Node};
use crate::core::mouse::{Cursor, Interaction};
use crate::core::renderer::Style;
use crate::core::widget::tree::{State, Tag};
use crate::core::widget::{tree, Tree};
use crate::core::{
    self, layout, mouse, widget, window, Clipboard, Element, Layout, Length,
    Rectangle, Shell, Size, Widget,
};
use std::marker::PhantomData;

mod event;
mod program;

pub use event::Event;
pub use iced_graphics::Transformation;
pub use iced_wgpu::custom::Primitive;
pub use iced_wgpu::custom::Storage;
pub use program::Program;

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

impl<P, Message, Theme> Widget<Message, crate::Renderer<Theme>>
    for Shader<Message, P>
where
    P: Program<Message>,
{
    fn tag(&self) -> Tag {
        struct Tag<T>(T);
        tree::Tag::of::<Tag<P::State>>()
    }

    fn state(&self) -> State {
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
        _tree: &mut Tree,
        _renderer: &crate::Renderer<Theme>,
        limits: &Limits,
    ) -> Node {
        let limits = limits.width(self.width).height(self.height);
        let size = limits.resolve(Size::ZERO);

        layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: crate::core::Event,
        layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &crate::Renderer<Theme>,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> Status {
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
            _ => None,
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
        cursor: Cursor,
        _viewport: &Rectangle,
        _renderer: &crate::Renderer<Theme>,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<P::State>();

        self.program.mouse_interaction(state, bounds, cursor)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut crate::Renderer<Theme>,
        _theme: &Theme,
        _style: &Style,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<P::State>();

        renderer.draw_custom(
            bounds,
            self.program.draw(state, cursor_position, bounds),
        );
    }
}

impl<'a, M, P, Theme> From<Shader<M, P>>
    for Element<'a, M, crate::Renderer<Theme>>
where
    M: 'a,
    P: Program<M> + 'a,
{
    fn from(custom: Shader<M, P>) -> Element<'a, M, crate::Renderer<Theme>> {
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
        cursor: Cursor,
        shell: &mut Shell<'_, Message>,
    ) -> (Status, Option<Message>) {
        T::update(self, state, event, bounds, cursor, shell)
    }

    fn draw(
        &self,
        state: &Self::State,
        cursor: Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        T::draw(self, state, cursor, bounds)
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Interaction {
        T::mouse_interaction(self, state, bounds, cursor)
    }
}
