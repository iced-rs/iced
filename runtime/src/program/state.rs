use crate::core::event::{self, Event};
use crate::core::mouse;
use crate::core::renderer;
use crate::core::{Clipboard, Point, Size};
use crate::user_interface::{self, UserInterface};
use crate::{Command, Debug, Program};

/// The execution state of a [`Program`]. It leverages caching, event
/// processing, and rendering primitive storage.
#[allow(missing_debug_implementations)]
pub struct State<P>
where
    P: Program + 'static,
{
    program: P,
    cache: Option<user_interface::Cache>,
    queued_events: Vec<Event>,
    queued_messages: Vec<P::Message>,
    mouse_interaction: mouse::Interaction,
}

impl<P> State<P>
where
    P: Program + 'static,
{
    /// Creates a new [`State`] with the provided [`Program`], initializing its
    /// primitive with the given logical bounds and renderer.
    pub fn new(
        id: crate::window::Id,
        mut program: P,
        bounds: Size,
        renderer: &mut P::Renderer,
        debug: &mut Debug,
    ) -> Self {
        let user_interface = build_user_interface(
            id,
            &mut program,
            user_interface::Cache::default(),
            renderer,
            bounds,
            debug,
        );

        let cache = Some(user_interface.into_cache());

        State {
            program,
            cache,
            queued_events: Vec::new(),
            queued_messages: Vec::new(),
            mouse_interaction: mouse::Interaction::Idle,
        }
    }

    /// Returns a reference to the [`Program`] of the [`State`].
    pub fn program(&self) -> &P {
        &self.program
    }

    /// Queues an event in the [`State`] for processing during an [`update`].
    ///
    /// [`update`]: Self::update
    pub fn queue_event(&mut self, event: Event) {
        self.queued_events.push(event);
    }

    /// Queues a message in the [`State`] for processing during an [`update`].
    ///
    /// [`update`]: Self::update
    pub fn queue_message(&mut self, message: P::Message) {
        self.queued_messages.push(message);
    }

    /// Returns whether the event queue of the [`State`] is empty or not.
    pub fn is_queue_empty(&self) -> bool {
        self.queued_events.is_empty() && self.queued_messages.is_empty()
    }

    /// Returns the current [`mouse::Interaction`] of the [`State`].
    pub fn mouse_interaction(&self) -> mouse::Interaction {
        self.mouse_interaction
    }

    /// Processes all the queued events and messages, rebuilding and redrawing
    /// the widgets of the linked [`Program`] if necessary.
    ///
    /// Returns a list containing the instances of [`Event`] that were not
    /// captured by any widget, and the [`Command`] obtained from [`Program`]
    /// after updating it, only if an update was necessary.
    pub fn update(
        &mut self,
        id: crate::window::Id,
        bounds: Size,
        cursor_position: Point,
        renderer: &mut P::Renderer,
        theme: &<P::Renderer as iced_core::Renderer>::Theme,
        style: &renderer::Style,
        clipboard: &mut dyn Clipboard,
        debug: &mut Debug,
    ) -> (Vec<Event>, Option<Command<P::Message>>) {
        let mut user_interface = build_user_interface(
            id,
            &mut self.program,
            self.cache.take().unwrap(),
            renderer,
            bounds,
            debug,
        );

        debug.event_processing_started();
        let mut messages = Vec::new();

        let (_, event_statuses) = user_interface.update(
            &self.queued_events,
            cursor_position,
            renderer,
            clipboard,
            &mut messages,
        );

        let uncaptured_events = self
            .queued_events
            .iter()
            .zip(event_statuses)
            .filter_map(|(event, status)| {
                matches!(status, event::Status::Ignored).then_some(event)
            })
            .cloned()
            .collect();

        self.queued_events.clear();
        messages.append(&mut self.queued_messages);
        debug.event_processing_finished();

        let command = if messages.is_empty() {
            debug.draw_started();
            self.mouse_interaction =
                user_interface.draw(renderer, theme, style, cursor_position);
            debug.draw_finished();

            self.cache = Some(user_interface.into_cache());

            None
        } else {
            // When there are messages, we are forced to rebuild twice
            // for now :^)
            let temp_cache = user_interface.into_cache();

            let commands =
                Command::batch(messages.into_iter().map(|message| {
                    debug.log_message(&message);

                    debug.update_started();
                    let command = self.program.update(message);
                    debug.update_finished();

                    command
                }));

            let mut user_interface = build_user_interface(
                id,
                &mut self.program,
                temp_cache,
                renderer,
                bounds,
                debug,
            );

            debug.draw_started();
            self.mouse_interaction =
                user_interface.draw(renderer, theme, style, cursor_position);
            debug.draw_finished();

            self.cache = Some(user_interface.into_cache());

            Some(commands)
        };

        (uncaptured_events, command)
    }
}

fn build_user_interface<'a, P: Program>(
    id: crate::window::Id,
    program: &'a mut P,
    cache: user_interface::Cache,
    renderer: &mut P::Renderer,
    size: Size,
    debug: &mut Debug,
) -> UserInterface<'a, P::Message, P::Renderer> {
    debug.view_started();
    let view = program.view(id);
    debug.view_finished();

    debug.layout_started();
    let user_interface = UserInterface::build(view, size, cache, renderer);
    debug.layout_finished();

    user_interface
}
