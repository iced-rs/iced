//! The internal state of a multi-window [`Program`].
use crate::core::event::{self, Event};
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget::operation::{self, Operation};
use crate::core::{Clipboard, Size};
use crate::user_interface::{self, UserInterface};
use crate::{Command, Debug, Program};

/// The execution state of a multi-window [`Program`]. It leverages caching, event
/// processing, and rendering primitive storage.
#[allow(missing_debug_implementations)]
pub struct State<P>
where
    P: Program + 'static,
{
    program: P,
    caches: Option<Vec<user_interface::Cache>>,
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
        program: P,
        bounds: Size,
        renderer: &mut P::Renderer,
        debug: &mut Debug,
    ) -> Self {
        let user_interface = build_user_interface(
            &program,
            user_interface::Cache::default(),
            renderer,
            bounds,
            debug,
        );

        let caches = Some(vec![user_interface.into_cache()]);

        State {
            program,
            caches,
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
        debug: &mut Debug,
    ) -> (Vec<Event>, Option<Command<P::Message>>) {
        let mut user_interfaces = build_user_interfaces(
            &self.program,
            self.caches.take().unwrap(),
            renderer,
            bounds,
            debug,
        );

        debug.event_processing_started();
        let mut messages = Vec::new();

        let uncaptured_events = user_interfaces.iter_mut().fold(
            vec![],
            |mut uncaptured_events, ui| {
                let (_, event_statuses) = ui.update(
                    &self.queued_events,
                    cursor,
                    renderer,
                    clipboard,
                    &mut messages,
                );

                uncaptured_events.extend(
                    self.queued_events
                        .iter()
                        .zip(event_statuses)
                        .filter_map(|(event, status)| {
                            matches!(status, event::Status::Ignored)
                                .then_some(event)
                        })
                        .cloned(),
                );
                uncaptured_events
            },
        );

        self.queued_events.clear();
        messages.append(&mut self.queued_messages);
        debug.event_processing_finished();

        let commands = if messages.is_empty() {
            debug.draw_started();

            for ui in &mut user_interfaces {
                self.mouse_interaction =
                    ui.draw(renderer, theme, style, cursor);
            }

            debug.draw_finished();

            self.caches = Some(
                user_interfaces
                    .drain(..)
                    .map(UserInterface::into_cache)
                    .collect(),
            );

            None
        } else {
            let temp_caches = user_interfaces
                .drain(..)
                .map(UserInterface::into_cache)
                .collect();

            drop(user_interfaces);

            let commands = Command::batch(messages.into_iter().map(|msg| {
                debug.log_message(&msg);

                debug.update_started();
                let command = self.program.update(msg);
                debug.update_finished();

                command
            }));

            let mut user_interfaces = build_user_interfaces(
                &self.program,
                temp_caches,
                renderer,
                bounds,
                debug,
            );

            debug.draw_started();
            for ui in &mut user_interfaces {
                self.mouse_interaction =
                    ui.draw(renderer, theme, style, cursor);
            }
            debug.draw_finished();

            self.caches = Some(
                user_interfaces
                    .drain(..)
                    .map(UserInterface::into_cache)
                    .collect(),
            );

            Some(commands)
        };

        (uncaptured_events, commands)
    }

    /// Applies widget [`Operation`]s to the [`State`].
    pub fn operate(
        &mut self,
        renderer: &mut P::Renderer,
        operations: impl Iterator<Item = Box<dyn Operation<P::Message>>>,
        bounds: Size,
        debug: &mut Debug,
    ) {
        let mut user_interfaces = build_user_interfaces(
            &self.program,
            self.caches.take().unwrap(),
            renderer,
            bounds,
            debug,
        );

        for operation in operations {
            let mut current_operation = Some(operation);

            while let Some(mut operation) = current_operation.take() {
                for ui in &mut user_interfaces {
                    ui.operate(renderer, operation.as_mut());
                }

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

        self.caches = Some(
            user_interfaces
                .drain(..)
                .map(UserInterface::into_cache)
                .collect(),
        );
    }
}

fn build_user_interfaces<'a, P: Program>(
    program: &'a P,
    mut caches: Vec<user_interface::Cache>,
    renderer: &mut P::Renderer,
    size: Size,
    debug: &mut Debug,
) -> Vec<UserInterface<'a, P::Message, P::Theme, P::Renderer>> {
    caches
        .drain(..)
        .map(|cache| {
            build_user_interface(program, cache, renderer, size, debug)
        })
        .collect()
}

fn build_user_interface<'a, P: Program>(
    program: &'a P,
    cache: user_interface::Cache,
    renderer: &mut P::Renderer,
    size: Size,
    debug: &mut Debug,
) -> UserInterface<'a, P::Message, P::Theme, P::Renderer> {
    debug.view_started();
    let view = program.view();
    debug.view_finished();

    debug.layout_started();
    let user_interface = UserInterface::build(view, size, cache, renderer);
    debug.layout_finished();

    user_interface
}
