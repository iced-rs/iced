use crate::Instruction;
use crate::core;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget;
use crate::core::window;
use crate::core::{Element, Size};
use crate::program;
use crate::program::Program;
use crate::runtime::futures::futures::StreamExt;
use crate::runtime::futures::futures::channel::mpsc;
use crate::runtime::futures::futures::stream;
use crate::runtime::futures::subscription;
use crate::runtime::futures::{Executor, Runtime};
use crate::runtime::task;
use crate::runtime::user_interface;
use crate::runtime::{Action, Task, UserInterface};

use std::fmt;

#[allow(missing_debug_implementations)]
pub struct Emulator<P: Program> {
    state: P::State,
    runtime: Runtime<P::Executor, mpsc::Sender<Event<P>>, Event<P>>,
    renderer: P::Renderer,
    mode: Mode,
    size: Size,
    window: window::Id,
    cursor: mouse::Cursor,
    clipboard: Clipboard,
    cache: Option<user_interface::Cache>,
}

#[allow(missing_debug_implementations)]
pub enum Event<P: Program> {
    Action(Action<P::Message>),
    Ready,
}

impl<P: Program + 'static> Emulator<P> {
    pub fn new(
        sender: mpsc::Sender<Event<P>>,
        program: &P,
        mode: Mode,
        size: Size,
    ) -> Emulator<P> {
        Self::with_preset(sender, program, mode, size, None)
    }

    pub fn with_preset(
        sender: mpsc::Sender<Event<P>>,
        program: &P,
        mode: Mode,
        size: Size,
        preset: Option<&program::Preset<P::State, P::Message>>,
    ) -> Emulator<P> {
        use renderer::Headless;

        let settings = program.settings();

        // TODO: Error handling
        let executor = P::Executor::new().expect("Create emulator executor");

        let renderer = executor
            .block_on(P::Renderer::new(
                settings.default_font,
                settings.default_text_size,
                None,
            ))
            .expect("Create emulator renderer");

        let runtime = Runtime::new(executor, sender);

        let (state, task) = if let Some(preset) = preset {
            preset.boot()
        } else {
            program.boot()
        };

        let mut emulator = Self {
            state,
            runtime,
            renderer,
            mode,
            size,
            clipboard: Clipboard { content: None },
            cursor: mouse::Cursor::Unavailable,
            window: window::Id::unique(),
            cache: Some(user_interface::Cache::default()),
        };

        emulator.wait_for(task);
        emulator.resubscribe(program);

        emulator
    }

    pub fn update(&mut self, program: &P, message: P::Message) {
        let task = program.update(&mut self.state, message);

        if let Some(stream) = task::into_stream(task) {
            self.runtime.run(stream.map(Event::Action).boxed());
        }

        self.resubscribe(program);
    }

    pub fn perform(&mut self, program: &P, action: Action<P::Message>) {
        match action {
            Action::Output(message) => {
                self.update(program, message);
            }
            Action::LoadFont { .. } => {
                // TODO
            }
            Action::Widget(operation) => {
                let mut user_interface = UserInterface::build(
                    program.view(&self.state, self.window),
                    self.size,
                    self.cache.take().unwrap(),
                    &mut self.renderer,
                );

                let mut operation = Some(operation);

                while let Some(mut current) = operation.take() {
                    user_interface.operate(&self.renderer, &mut current);

                    match current.finish() {
                        widget::operation::Outcome::None => {}
                        widget::operation::Outcome::Some(()) => {}
                        widget::operation::Outcome::Chain(next) => {
                            operation = Some(next);
                        }
                    }
                }

                self.cache = Some(user_interface.into_cache());
            }
            Action::Clipboard(action) => {
                // TODO
                dbg!(action);
            }
            Action::Window(_action) => {
                // TODO
            }
            Action::System(action) => {
                // TODO
                dbg!(action);
            }
            Action::Exit => {
                // TODO
            }
        }
    }

    pub fn run(&mut self, program: &P, instruction: Instruction) {
        let mut user_interface = UserInterface::build(
            program.view(&self.state, self.window),
            self.size,
            self.cache.take().unwrap(),
            &mut self.renderer,
        );

        let mut messages = Vec::new();

        match instruction {
            Instruction::Interact(interaction) => {
                let events = interaction.events();

                for event in &events {
                    if let core::Event::Mouse(mouse::Event::CursorMoved {
                        position,
                    }) = event
                    {
                        self.cursor = mouse::Cursor::Available(*position);
                    }
                }

                let (_state, _status) = user_interface.update(
                    &events,
                    self.cursor,
                    &mut self.renderer,
                    &mut self.clipboard,
                    &mut messages,
                );
            }
        }

        self.cache = Some(user_interface.into_cache());

        let task = Task::batch(
            messages
                .into_iter()
                .map(|message| program.update(&mut self.state, message)),
        );

        self.wait_for(task);
        self.resubscribe(program);
    }

    pub fn wait_for(&mut self, task: Task<P::Message>) {
        if let Some(stream) = task::into_stream(task) {
            match self.mode {
                Mode::Patient => {
                    self.runtime.run(
                        stream
                            .map(Event::Action)
                            .chain(stream::once(async { Event::Ready }))
                            .boxed(),
                    );
                }
                Mode::Impatient => {
                    self.runtime.run(stream.map(Event::Action).boxed());
                    self.runtime.send(Event::Ready);
                }
            }
        } else {
            self.runtime.send(Event::Ready);
        }
    }

    pub fn resubscribe(&mut self, program: &P) {
        self.runtime.track(subscription::into_recipes(
            program
                .subscription(&self.state)
                .map(|message| Event::Action(Action::Output(message))),
        ));
    }

    pub fn view(
        &self,
        program: &P,
    ) -> Element<'_, P::Message, P::Theme, P::Renderer> {
        program.view(&self.state, self.window)
    }

    pub fn theme(&self, program: &P) -> P::Theme {
        program.theme(&self.state, self.window)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    Patient,
    #[default]
    Impatient,
}

impl Mode {
    pub const ALL: &[Self] = &[Self::Patient, Self::Impatient];
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Patient => f.write_str("Patient"),
            Mode::Impatient => f.write_str("Impatient"),
        }
    }
}

struct Clipboard {
    content: Option<String>,
}

impl core::Clipboard for Clipboard {
    fn read(&self, _kind: core::clipboard::Kind) -> Option<String> {
        self.content.clone()
    }

    fn write(&mut self, _kind: core::clipboard::Kind, contents: String) {
        self.content = Some(contents);
    }
}
