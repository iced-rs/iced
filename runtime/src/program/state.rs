use crate::core::event::{self, Event};
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget::operation::{self, Operation};
use crate::core::{Clipboard, Size};
use crate::debug;
use crate::user_interface::{self, UserInterface};
use crate::{Command, Program};

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
        mut program: P,
        bounds: Size,
        renderer: &mut P::Renderer,
    ) -> Self {
        let user_interface = build_user_interface(
            &mut program,
            user_interface::Cache::default(),
            renderer,
            bounds,
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
        bounds: Size,
        cursor: mouse::Cursor,
        renderer: &mut P::Renderer,
        theme: &P::Theme,
        style: &renderer::Style,
        clipboard: &mut dyn Clipboard,
    ) -> (Vec<Event>, Option<Command<P::Message>>) {
        let mut user_interface = build_user_interface(
            &mut self.program,
            self.cache.take().unwrap(),
            renderer,
            bounds,
        );

        let interact_timer = debug::interact_time();
        let mut messages = Vec::new();

        let (_, event_statuses) = user_interface.update(
            &self.queued_events,
            cursor,
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
        drop(interact_timer);

        let command = if messages.is_empty() {
            let draw_timer = debug::draw_time();
            self.mouse_interaction =
                user_interface.draw(renderer, theme, style, cursor);
            drop(draw_timer);

            self.cache = Some(user_interface.into_cache());

            None
        } else {
            // When there are messages, we are forced to rebuild twice
            // for now :^)
            let temp_cache = user_interface.into_cache();

            let commands =
                Command::batch(messages.into_iter().map(|message| {
                    debug::log_message(&message);

                    let update_timer = debug::update_time();
                    let command = self.program.update(message);
                    drop(update_timer);

                    command
                }));

            let mut user_interface = build_user_interface(
                &mut self.program,
                temp_cache,
                renderer,
                bounds,
            );

            let draw_timer = debug::draw_time();
            self.mouse_interaction =
                user_interface.draw(renderer, theme, style, cursor);
            drop(draw_timer);

            self.cache = Some(user_interface.into_cache());

            Some(commands)
        };

        (uncaptured_events, command)
    }

    /// Applies [`Operation`]s to the [`State`]
    pub fn operate(
        &mut self,
        renderer: &mut P::Renderer,
        operations: impl Iterator<Item = Box<dyn Operation<P::Message>>>,
        bounds: Size,
    ) {
        let mut user_interface = build_user_interface(
            &mut self.program,
            self.cache.take().unwrap(),
            renderer,
            bounds,
        );

        for operation in operations {
            let mut current_operation = Some(operation);

            while let Some(mut operation) = current_operation.take() {
                user_interface.operate(renderer, operation.as_mut());

                match operation.finish() {
                    operation::Outcome::None => {}
                    operation::Outcome::Some(message) => {
                        self.queued_messages.push(message);
                    }
                    operation::Outcome::Chain(next) => {
                        current_operation = Some(next);
                    }
                };
            }
        }

        self.cache = Some(user_interface.into_cache());
    }
}

fn build_user_interface<'a, P: Program>(
    program: &'a mut P,
    cache: user_interface::Cache,
    renderer: &mut P::Renderer,
    size: Size,
) -> UserInterface<'a, P::Message, P::Theme, P::Renderer> {
    let view_timer = debug::view_time();
    let view = program.view();
    drop(view_timer);

    let layout_timer = debug::layout_time();
    let user_interface = UserInterface::build(view, size, cache, renderer);
    drop(layout_timer);

    user_interface
}
