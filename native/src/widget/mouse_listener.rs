//! A container for capturing mouse events.

use crate::event::{self, Event};
use crate::layout;
use crate::mouse;
use crate::overlay;
use crate::renderer;
use crate::touch;
use crate::widget::{tree, Operation, Tree};
use crate::{
    Clipboard, Element, Layout, Length, Point, Rectangle, Shell, Size, Widget,
};

use std::u32;

/// Emit messages on mouse events.
#[allow(missing_debug_implementations)]
pub struct MouseListener<'a, Message, Renderer> {
    content: Element<'a, Message, Renderer>,

    /// Sets the message to emit on a left mouse button press.
    on_press: Option<Message>,

    /// Sets the message to emit on a left mouse button release.
    on_release: Option<Message>,

    /// Sets the message to emit on a right mouse button press.
    on_right_press: Option<Message>,

    /// Sets the message to emit on a right mouse button release.
    on_right_release: Option<Message>,

    /// Sets the message to emit on a middle mouse button press.
    on_middle_press: Option<Message>,

    /// Sets the message to emit on a middle mouse button release.
    on_middle_release: Option<Message>,

    /// Sets the message to emit when the mouse enters the widget.
    on_mouse_enter: Option<Message>,

    /// Sets the messsage to emit when the mouse exits the widget.
    on_mouse_exit: Option<Message>,
}

impl<'a, Message, Renderer> MouseListener<'a, Message, Renderer> {
    /// The message to emit on a left button press.
    #[must_use]
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// The message to emit on a left button release.
    #[must_use]
    pub fn on_release(mut self, message: Message) -> Self {
        self.on_release = Some(message);
        self
    }

    /// The message to emit on a right button press.
    #[must_use]
    pub fn on_right_press(mut self, message: Message) -> Self {
        self.on_right_press = Some(message);
        self
    }

    /// The message to emit on a right button release.
    #[must_use]
    pub fn on_right_release(mut self, message: Message) -> Self {
        self.on_right_release = Some(message);
        self
    }

    /// The message to emit on a middle button press.
    #[must_use]
    pub fn on_middle_press(mut self, message: Message) -> Self {
        self.on_middle_press = Some(message);
        self
    }

    /// The message to emit on a middle button release.
    #[must_use]
    pub fn on_middle_release(mut self, message: Message) -> Self {
        self.on_middle_release = Some(message);
        self
    }

    /// The message to emit when the mouse enters the widget.
    #[must_use]
    pub fn on_mouse_enter(mut self, message: Message) -> Self {
        self.on_mouse_enter = Some(message);
        self
    }

    /// The messsage to emit when the mouse exits the widget.
    #[must_use]
    pub fn on_mouse_exit(mut self, message: Message) -> Self {
        self.on_mouse_exit = Some(message);
        self
    }
}

/// Local state of the [`MouseListener`].
#[derive(Default)]
struct State {
    hovered: bool,
}

impl<'a, Message, Renderer> MouseListener<'a, Message, Renderer> {
    /// Creates an empty [`MouseListener`].
    pub fn new(content: impl Into<Element<'a, Message, Renderer>>) -> Self {
        MouseListener {
            content: content.into(),
            on_press: None,
            on_release: None,
            on_right_press: None,
            on_right_release: None,
            on_middle_press: None,
            on_middle_release: None,
            on_mouse_enter: None,
            on_mouse_exit: None,
        }
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for MouseListener<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
    Message: Clone,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn width(&self) -> Length {
        self.content.as_widget().width()
    }

    fn height(&self) -> Length {
        self.content.as_widget().height()
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout(
            renderer,
            limits,
            Widget::<Message, Renderer>::width(self),
            Widget::<Message, Renderer>::height(self),
            u32::MAX,
            u32::MAX,
            |renderer, limits| {
                self.content.as_widget().layout(renderer, limits)
            },
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(None, &mut |operation| {
            self.content.as_widget().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                operation,
            );
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        if let event::Status::Captured = self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout.children().next().unwrap(),
            cursor_position,
            renderer,
            clipboard,
            shell,
        ) {
            return event::Status::Captured;
        }

        update(
            self,
            &event,
            layout,
            cursor_position,
            shell,
            tree.state.downcast_mut::<State>(),
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            renderer_style,
            layout.children().next().unwrap(),
            cursor_position,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        self.content.as_widget().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
        )
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }
}

impl<'a, Message, Renderer> From<MouseListener<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + crate::Renderer,
{
    fn from(
        listener: MouseListener<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(listener)
    }
}

/// Processes the given [`Event`] and updates the [`State`] of an [`MouseListener`]
/// accordingly.
fn update<Message: Clone, Renderer>(
    widget: &mut MouseListener<'_, Message, Renderer>,
    event: &Event,
    layout: Layout<'_>,
    cursor_position: Point,
    shell: &mut Shell<'_, Message>,
    state: &mut State,
) -> event::Status {
    let hovered = state.hovered;

    if !layout.bounds().contains(cursor_position) {
        if hovered {
            state.hovered = false;
            if let Some(message) = widget.on_mouse_exit.clone() {
                shell.publish(message);
                return event::Status::Captured;
            }
        }

        return event::Status::Ignored;
    }

    state.hovered = true;

    if !hovered {
        if let Some(message) = widget.on_mouse_enter.clone() {
            shell.publish(message);
            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_press.clone() {
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) = event
        {
            shell.publish(message);
            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_release.clone() {
        if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerLifted { .. }) = event
        {
            shell.publish(message);
            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_right_press.clone() {
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) =
            event
        {
            shell.publish(message);
            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_right_release.clone() {
        if let Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Right,
        )) = event
        {
            shell.publish(message);
            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_middle_press.clone() {
        if let Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Middle,
        )) = event
        {
            shell.publish(message);
            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_middle_release.clone() {
        if let Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Middle,
        )) = event
        {
            shell.publish(message);
            return event::Status::Captured;
        }
    }

    event::Status::Ignored
}

/// Computes the layout of a [`MouseListener`].
pub fn layout<Renderer>(
    renderer: &Renderer,
    limits: &layout::Limits,
    width: Length,
    height: Length,
    max_height: u32,
    max_width: u32,
    layout_content: impl FnOnce(&Renderer, &layout::Limits) -> layout::Node,
) -> layout::Node {
    let limits = limits.loose().max_height(max_height).max_width(max_width).width(width).height(height);

    let content = layout_content(renderer, &limits);
    let size = limits.resolve(content.size());

    layout::Node::with_children(size, vec![content])
}
