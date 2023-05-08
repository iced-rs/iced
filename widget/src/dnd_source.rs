//! A widget that can be dragged and dropped.

use sctk::reexports::client::protocol::wl_data_device_manager::DndAction;

use crate::core::{
    event, layout, mouse, overlay, touch, Clipboard, Element, Event, Length,
    Point, Rectangle, Shell, Widget,
};

use crate::core::widget::{
    operation::OperationOutputWrapper, tree, Operation, Tree,
};

/// A widget that can be dragged and dropped.
#[allow(missing_debug_implementations)]
pub struct DndSource<'a, Message, Renderer> {
    content: Element<'a, Message, Renderer>,

    on_drag: Option<Message>,

    on_cancelled: Option<Message>,

    on_finished: Option<Message>,

    on_dropped: Option<Message>,

    on_selection_action: Option<Box<dyn Fn(DndAction) -> Message + 'a>>,

    drag_threshold: f32,

    /// Whether or not captured events should be handled by the widget.
    handle_captured_events: bool,
}

impl<'a, Message, Renderer> DndSource<'a, Message, Renderer> {
    /// The message to produce when the drag starts.
    #[must_use]
    pub fn on_drag(mut self, message: Message) -> Self {
        self.on_drag = Some(message);
        self
    }

    /// The message to produce when the drag is cancelled.
    #[must_use]
    pub fn on_cancelled(mut self, message: Message) -> Self {
        self.on_cancelled = Some(message);
        self
    }

    /// The message to produce when the drag is finished.
    #[must_use]
    pub fn on_finished(mut self, message: Message) -> Self {
        self.on_finished = Some(message);
        self
    }

    /// The message to produce when the drag is dropped.
    #[must_use]
    pub fn on_dropped(mut self, message: Message) -> Self {
        self.on_dropped = Some(message);
        self
    }

    /// The message to produce when the selection action is triggered.
    #[must_use]
    pub fn on_selection_action<F>(mut self, f: F) -> Self
    where
        F: Fn(DndAction) -> Message + 'a,
    {
        self.on_selection_action = Some(Box::new(f));
        self
    }

    /// The drag radius threshold.
    /// if the mouse is moved more than this radius while pressed, the drag event is triggered
    #[must_use]
    pub fn drag_threshold(mut self, radius: f32) -> Self {
        self.drag_threshold = radius.powi(2);
        self
    }

    /// Whether or not captured events should be handled by the widget.
    #[must_use]
    pub fn handle_captured_events(
        mut self,
        handle_captured_events: bool,
    ) -> Self {
        self.handle_captured_events = handle_captured_events;
        self
    }
}

/// Local state of the [`MouseListener`].
#[derive(Default)]
struct State {
    hovered: bool,
    left_pressed_position: Option<Point>,
    is_dragging: bool,
}

impl<'a, Message, Renderer> DndSource<'a, Message, Renderer> {
    /// Creates a new [`DndSource`].
    #[must_use]
    pub fn new(content: impl Into<Element<'a, Message, Renderer>>) -> Self {
        Self {
            content: content.into(),
            on_drag: None,
            on_cancelled: None,
            on_finished: None,
            on_dropped: None,
            on_selection_action: None,
            drag_threshold: 25.0,
            handle_captured_events: true,
        }
    }
}

impl<'a, Message, Renderer> From<DndSource<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: crate::core::Renderer + 'a,
    Message: Clone + 'a,
{
    fn from(
        dnd_source: DndSource<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(dnd_source)
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for DndSource<'a, Message, Renderer>
where
    Renderer: crate::core::Renderer,
    Message: Clone,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.content));
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

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as crate::core::Renderer>::Theme,
        renderer_style: &crate::core::renderer::Style,
        layout: crate::core::Layout<'_>,
        cursor_position: Point,
        viewport: &crate::core::Rectangle,
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

    fn operate(
        &self,
        tree: &mut Tree,
        layout: layout::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<OperationOutputWrapper<Message>>,
    ) {
        operation.container(None, &mut |operation| {
            self.content.as_widget().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: layout::Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        self.content.as_widget_mut().overlay(
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

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: layout::Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let captured = self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout.children().next().unwrap(),
            cursor_position,
            renderer,
            clipboard,
            shell,
        );

        if captured == event::Status::Captured && !self.handle_captured_events {
            return event::Status::Captured;
        }

        let mut state = tree.state.downcast_mut::<State>();

        if matches!(
            event,
            Event::PlatformSpecific(event::PlatformSpecific::Wayland(
                event::wayland::Event::Seat(
                    event::wayland::SeatEvent::Leave,
                    _
                )
            )) | Event::Mouse(mouse::Event::ButtonReleased(
                mouse::Button::Left
            )) | Event::Touch(touch::Event::FingerLifted { .. })
                | Event::Touch(touch::Event::FingerLost { .. })
        ) {
            state.left_pressed_position = None;
            return event::Status::Captured;
        }

        if state.is_dragging {
            if let Event::PlatformSpecific(event::PlatformSpecific::Wayland(
                event::wayland::Event::DataSource(
                    event::wayland::DataSourceEvent::Cancelled,
                ),
            )) = event
            {
                if let Some(on_cancelled) = self.on_cancelled.clone() {
                    state.is_dragging = false;
                    shell.publish(on_cancelled);
                    return event::Status::Captured;
                }
            }

            if let Event::PlatformSpecific(event::PlatformSpecific::Wayland(
                event::wayland::Event::DataSource(
                    event::wayland::DataSourceEvent::DndFinished,
                ),
            )) = event
            {
                if let Some(on_finished) = self.on_finished.clone() {
                    state.is_dragging = false;
                    shell.publish(on_finished);
                    return event::Status::Captured;
                }
            }

            if let Event::PlatformSpecific(event::PlatformSpecific::Wayland(
                event::wayland::Event::DataSource(
                    event::wayland::DataSourceEvent::DndDropPerformed,
                ),
            )) = event
            {
                if let Some(on_dropped) = self.on_dropped.clone() {
                    shell.publish(on_dropped);
                    return event::Status::Captured;
                }
            }

            if let Event::PlatformSpecific(event::PlatformSpecific::Wayland(
                event::wayland::Event::DataSource(
                    event::wayland::DataSourceEvent::DndActionAccepted(action),
                ),
            )) = event
            {
                if let Some(on_action) = self.on_selection_action.as_deref() {
                    shell.publish(on_action(action));
                    return event::Status::Captured;
                }
            }
        }

        if cursor_position.x > 0.0
            && cursor_position.y > 0.0
            && !layout.bounds().contains(cursor_position)
        {
            // XXX if the widget is not hovered but the mouse is pressed,
            // we are triggering on_drag
            if let (Some(on_drag), Some(_)) =
                (self.on_drag.clone(), state.left_pressed_position.take())
            {
                shell.publish(on_drag);
                state.is_dragging = true;
                return event::Status::Captured;
            };
            return captured;
        }

        state.hovered = true;
        if let (Some(on_drag), Some(pressed_pos)) =
            (self.on_drag.clone(), state.left_pressed_position.clone())
        {
            if cursor_position.x < 0.0 || cursor_position.y < 0.0 {
                return captured;
            }
            let distance = (cursor_position.x - pressed_pos.x).powi(2)
                + (cursor_position.y - pressed_pos.y).powi(2);
            if distance > self.drag_threshold {
                state.left_pressed_position = None;
                state.is_dragging = true;
                shell.publish(on_drag);
                return event::Status::Captured;
            }
        }

        if self.on_drag.is_some() {
            if let Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left,
            ))
            | Event::Touch(touch::Event::FingerPressed { .. }) = event
            {
                state.left_pressed_position = Some(cursor_position);
                return event::Status::Captured;
            }
        }

        captured
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: layout::Layout<'_>,
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
}

/// Computes the layout of a [`DndSource`].
pub fn layout<Renderer>(
    renderer: &Renderer,
    limits: &layout::Limits,
    width: Length,
    height: Length,
    max_height: u32,
    max_width: u32,
    layout_content: impl FnOnce(&Renderer, &layout::Limits) -> layout::Node,
) -> layout::Node {
    let limits = limits
        .loose()
        .max_height(max_height as f32)
        .max_width(max_width as f32)
        .width(width)
        .height(height);

    let content = layout_content(renderer, &limits);
    let size = limits.resolve(content.size());

    layout::Node::with_children(size, vec![content])
}
