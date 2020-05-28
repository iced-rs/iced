use crate::{
    Cache, Clipboard, Command, Debug, Event, Program, Renderer, Size,
    UserInterface,
};

/// The execution state of a [`Program`]. It leverages caching, event
/// processing, and rendering primitive storage.
///
/// [`Program`]: trait.Program.html
#[allow(missing_debug_implementations)]
pub struct State<P>
where
    P: Program + 'static,
{
    program: P,
    cache: Option<Cache>,
    primitive: <P::Renderer as Renderer>::Output,
    queued_events: Vec<Event>,
    queued_messages: Vec<P::Message>,
}

impl<P> State<P>
where
    P: Program + 'static,
{
    /// Creates a new [`State`] with the provided [`Program`], initializing its
    /// primitive with the given logical bounds and renderer.
    ///
    /// [`State`]: struct.State.html
    /// [`Program`]: trait.Program.html
    pub fn new(
        mut program: P,
        bounds: Size,
        renderer: &mut P::Renderer,
        debug: &mut Debug,
    ) -> Self {
        let user_interface = build_user_interface(
            &mut program,
            Cache::default(),
            renderer,
            bounds,
            debug,
        );

        debug.draw_started();
        let primitive = user_interface.draw(renderer);
        debug.draw_finished();

        let cache = Some(user_interface.into_cache());

        State {
            program,
            cache,
            primitive,
            queued_events: Vec::new(),
            queued_messages: Vec::new(),
        }
    }

    /// Returns a reference to the [`Program`] of the [`State`].
    ///
    /// [`Program`]: trait.Program.html
    /// [`State`]: struct.State.html
    pub fn program(&self) -> &P {
        &self.program
    }

    /// Returns a reference to the current rendering primitive of the [`State`].
    ///
    /// [`State`]: struct.State.html
    pub fn primitive(&self) -> &<P::Renderer as Renderer>::Output {
        &self.primitive
    }

    /// Queues an event in the [`State`] for processing during an [`update`].
    ///
    /// [`State`]: struct.State.html
    /// [`update`]: #method.update
    pub fn queue_event(&mut self, event: Event) {
        self.queued_events.push(event);
    }

    /// Queues a message in the [`State`] for processing during an [`update`].
    ///
    /// [`State`]: struct.State.html
    /// [`update`]: #method.update
    pub fn queue_message(&mut self, message: P::Message) {
        self.queued_messages.push(message);
    }

    /// Processes all the queued events and messages, rebuilding and redrawing
    /// the widgets of the linked [`Program`] if necessary.
    ///
    /// Returns the [`Command`] obtained from [`Program`] after updating it,
    /// only if an update was necessary.
    ///
    /// [`Program`]: trait.Program.html
    pub fn update(
        &mut self,
        clipboard: Option<&dyn Clipboard>,
        bounds: Size,
        renderer: &mut P::Renderer,
        debug: &mut Debug,
    ) -> Option<Command<P::Message>> {
        if self.queued_events.is_empty() && self.queued_messages.is_empty() {
            return None;
        }

        let mut user_interface = build_user_interface(
            &mut self.program,
            self.cache.take().unwrap(),
            renderer,
            bounds,
            debug,
        );

        debug.event_processing_started();
        let mut messages = user_interface.update(
            self.queued_events.drain(..),
            clipboard,
            renderer,
        );
        messages.extend(self.queued_messages.drain(..));
        debug.event_processing_finished();

        if messages.is_empty() {
            debug.draw_started();
            self.primitive = user_interface.draw(renderer);
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

            let user_interface = build_user_interface(
                &mut self.program,
                temp_cache,
                renderer,
                bounds,
                debug,
            );

            debug.draw_started();
            self.primitive = user_interface.draw(renderer);
            debug.draw_finished();

            self.cache = Some(user_interface.into_cache());

            Some(commands)
        }
    }
}

fn build_user_interface<'a, P: Program>(
    program: &'a mut P,
    cache: Cache,
    renderer: &mut P::Renderer,
    size: Size,
    debug: &mut Debug,
) -> UserInterface<'a, P::Message, P::Renderer> {
    debug.view_started();
    let view = program.view();
    debug.view_finished();

    debug.layout_started();
    let user_interface = UserInterface::build(view, size, cache, renderer);
    debug.layout_finished();

    user_interface
}
