//! Run your application in a headless runtime.
use crate::core;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::time::Instant;
use crate::core::widget;
use crate::core::window;
use crate::core::{Bytes, Element, Point, Size};
use crate::instruction;
use crate::program;
use crate::program::Program;
use crate::runtime;
use crate::runtime::futures::futures::StreamExt;
use crate::runtime::futures::futures::channel::mpsc;
use crate::runtime::futures::futures::stream;
use crate::runtime::futures::subscription;
use crate::runtime::futures::{Executor, Runtime};
use crate::runtime::task;
use crate::runtime::user_interface;
use crate::runtime::{Task, UserInterface};
use crate::{Instruction, Selector};

use std::fmt;

/// A headless runtime that can run iced applications and execute
/// [instructions](crate::Instruction).
///
/// An [`Emulator`] runs its program as faithfully as possible to the real thing.
/// It will run subscriptions and tasks with the [`Executor`](Program::Executor) of
/// the [`Program`].
///
/// If you want to run a simulation without side effects, use a [`Simulator`](crate::Simulator)
/// instead.
pub struct Emulator<P: Program> {
    state: P::State,
    runtime: Runtime<P::Executor, mpsc::Sender<Event<P>>, Event<P>>,
    renderer: P::Renderer,
    mode: Mode,
    size: Size,
    window: core::window::Id,
    cursor: mouse::Cursor,
    clipboard: Clipboard,
    cache: Option<user_interface::Cache>,
    pending_tasks: usize,
}

/// An emulation event.
pub enum Event<P: Program> {
    /// An action that must be [performed](Emulator::perform) by the [`Emulator`].
    Action(Action<P>),
    /// An [`Instruction`] failed to be executed.
    Failed(Instruction),
    /// The [`Emulator`] is ready.
    Ready,
}

/// An action that must be [performed](Emulator::perform) by the [`Emulator`].
pub struct Action<P: Program>(Action_<P>);

enum Action_<P: Program> {
    Runtime(runtime::Action<P::Message>),
    CountDown,
}

impl<P: Program + 'static> Emulator<P> {
    /// Creates a new [`Emulator`] of the [`Program`] with the given [`Mode`] and [`Size`].
    ///
    /// The [`Emulator`] will send [`Event`] notifications through the provided [`mpsc::Sender`].
    ///
    /// When the [`Emulator`] has finished booting, an [`Event::Ready`] will be produced.
    pub fn new(
        sender: mpsc::Sender<Event<P>>,
        program: &P,
        mode: Mode,
        size: Size,
    ) -> Emulator<P> {
        Self::with_preset(sender, program, mode, size, None)
    }

    /// Creates a new [`Emulator`] analogously to [`new`](Self::new), but it also takes a
    /// [`program::Preset`] that will be used as the initial state.
    ///
    /// When the [`Emulator`] has finished booting, an [`Event::Ready`] will be produced.
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

        let (state, task) = runtime.enter(|| {
            if let Some(preset) = preset {
                preset.boot()
            } else {
                program.boot()
            }
        });

        let mut emulator = Self {
            state,
            runtime,
            renderer,
            mode,
            size,
            clipboard: Clipboard { content: None },
            cursor: mouse::Cursor::Unavailable,
            window: core::window::Id::unique(),
            cache: Some(user_interface::Cache::default()),
            pending_tasks: 0,
        };

        emulator.resubscribe(program);
        emulator.wait_for(task);

        emulator
    }

    /// Updates the state of the [`Emulator`] program.
    ///
    /// This is equivalent to calling the [`Program::update`] function,
    /// resubscribing to any subscriptions, and running the resulting tasks
    /// concurrently.
    pub fn update(&mut self, program: &P, message: P::Message) {
        let task = self
            .runtime
            .enter(|| program.update(&mut self.state, message));

        self.resubscribe(program);

        match self.mode {
            Mode::Zen if self.pending_tasks > 0 => self.wait_for(task),
            _ => {
                if let Some(stream) = task::into_stream(task) {
                    self.runtime.run(
                        stream
                            .map(Action_::Runtime)
                            .map(Action)
                            .map(Event::Action)
                            .boxed(),
                    );
                }
            }
        }
    }

    /// Performs an [`Action`].
    ///
    /// Whenever an [`Emulator`] sends an [`Event::Action`], this
    /// method must be called to proceed with the execution.
    pub fn perform(&mut self, program: &P, action: Action<P>) {
        match action.0 {
            Action_::CountDown => {
                if self.pending_tasks > 0 {
                    self.pending_tasks -= 1;

                    if self.pending_tasks == 0 {
                        self.runtime.send(Event::Ready);
                    }
                }
            }
            Action_::Runtime(action) => match action {
                runtime::Action::Output(message) => {
                    self.update(program, message);
                }
                runtime::Action::LoadFont { .. } => {
                    // TODO
                }
                runtime::Action::Widget(operation) => {
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
                runtime::Action::Clipboard(action) => {
                    // TODO
                    dbg!(action);
                }
                runtime::Action::Window(action) => {
                    use crate::runtime::window;

                    match action {
                        window::Action::Open(id, _settings, sender) => {
                            self.window = id;

                            let _ = sender.send(self.window);
                        }
                        window::Action::GetOldest(sender)
                        | window::Action::GetLatest(sender) => {
                            let _ = sender.send(Some(self.window));
                        }
                        window::Action::GetSize(id, sender) => {
                            if id == self.window {
                                let _ = sender.send(self.size);
                            }
                        }
                        window::Action::GetMaximized(id, sender) => {
                            if id == self.window {
                                let _ = sender.send(false);
                            }
                        }
                        window::Action::GetMinimized(id, sender) => {
                            if id == self.window {
                                let _ = sender.send(None);
                            }
                        }
                        window::Action::GetPosition(id, sender) => {
                            if id == self.window {
                                let _ = sender.send(Some(Point::ORIGIN));
                            }
                        }
                        window::Action::GetScaleFactor(id, sender) => {
                            if id == self.window {
                                let _ = sender.send(1.0);
                            }
                        }
                        window::Action::GetMode(id, sender) => {
                            if id == self.window {
                                let _ =
                                    sender.send(core::window::Mode::Windowed);
                            }
                        }
                        _ => {
                            // Ignored
                        }
                    }
                }
                runtime::Action::System(action) => {
                    // TODO
                    dbg!(action);
                }
                iced_runtime::Action::Image(action) => {
                    // TODO
                    dbg!(action);
                }
                runtime::Action::Exit => {
                    // TODO
                }
                runtime::Action::Reload => {
                    // TODO
                }
            },
        }
    }

    /// Runs an [`Instruction`].
    ///
    /// If the [`Instruction`] executes successfully, an [`Event::Ready`] will be
    /// produced by the [`Emulator`].
    ///
    /// Otherwise, an [`Event::Failed`] will be triggered.
    pub fn run(&mut self, program: &P, instruction: Instruction) {
        let mut user_interface = UserInterface::build(
            program.view(&self.state, self.window),
            self.size,
            self.cache.take().unwrap(),
            &mut self.renderer,
        );

        let mut messages = Vec::new();

        match &instruction {
            Instruction::Interact(interaction) => {
                let Some(events) = interaction.events(|target| match target {
                    instruction::Target::Point(position) => Some(*position),
                    instruction::Target::Text(text) => {
                        use widget::Operation;

                        let mut operation = Selector::find(text.as_str());

                        user_interface.operate(
                            &self.renderer,
                            &mut widget::operation::black_box(&mut operation),
                        );

                        match operation.finish() {
                            widget::operation::Outcome::Some(text) => {
                                Some(text?.visible_bounds()?.center())
                            }
                            _ => None,
                        }
                    }
                }) else {
                    self.runtime.send(Event::Failed(instruction));
                    self.cache = Some(user_interface.into_cache());
                    return;
                };

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

                self.cache = Some(user_interface.into_cache());

                let task = self.runtime.enter(|| {
                    Task::batch(messages.into_iter().map(|message| {
                        program.update(&mut self.state, message)
                    }))
                });

                self.resubscribe(program);
                self.wait_for(task);
            }
            Instruction::Expect(expectation) => match expectation {
                instruction::Expectation::Text(text) => {
                    use widget::Operation;

                    let mut operation = Selector::find(text.as_str());

                    user_interface.operate(
                        &self.renderer,
                        &mut widget::operation::black_box(&mut operation),
                    );

                    match operation.finish() {
                        widget::operation::Outcome::Some(Some(_text)) => {
                            self.runtime.send(Event::Ready);
                        }
                        _ => {
                            self.runtime.send(Event::Failed(instruction));
                        }
                    }

                    self.cache = Some(user_interface.into_cache());
                }
            },
        }
    }

    fn wait_for(&mut self, task: Task<P::Message>) {
        if let Some(stream) = task::into_stream(task) {
            match self.mode {
                Mode::Zen => {
                    self.pending_tasks += 1;

                    self.runtime.run(
                        stream
                            .map(Action_::Runtime)
                            .map(Action)
                            .map(Event::Action)
                            .chain(stream::once(async {
                                Event::Action(Action(Action_::CountDown))
                            }))
                            .boxed(),
                    );
                }
                Mode::Patient => {
                    self.runtime.run(
                        stream
                            .map(Action_::Runtime)
                            .map(Action)
                            .map(Event::Action)
                            .chain(stream::once(async { Event::Ready }))
                            .boxed(),
                    );
                }
                Mode::Immediate => {
                    self.runtime.run(
                        stream
                            .map(Action_::Runtime)
                            .map(Action)
                            .map(Event::Action)
                            .boxed(),
                    );
                    self.runtime.send(Event::Ready);
                }
            }
        } else if self.pending_tasks == 0 {
            self.runtime.send(Event::Ready);
        }
    }

    fn resubscribe(&mut self, program: &P) {
        self.runtime
            .track(subscription::into_recipes(self.runtime.enter(|| {
                program.subscription(&self.state).map(|message| {
                    Event::Action(Action(Action_::Runtime(
                        runtime::Action::Output(message),
                    )))
                })
            })));
    }

    /// Returns the current view of the [`Emulator`].
    pub fn view(
        &self,
        program: &P,
    ) -> Element<'_, P::Message, P::Theme, P::Renderer> {
        program.view(&self.state, self.window)
    }

    /// Returns the current theme of the [`Emulator`].
    pub fn theme(&self, program: &P) -> Option<P::Theme> {
        program.theme(&self.state, self.window)
    }

    /// Takes a [`window::Screenshot`] of the current state of the [`Emulator`].
    pub fn screenshot(
        &mut self,
        program: &P,
        theme: &P::Theme,
        scale_factor: f32,
    ) -> window::Screenshot {
        use core::renderer::Headless;

        let style = program.style(&self.state, theme);

        let mut user_interface = UserInterface::build(
            program.view(&self.state, self.window),
            self.size,
            self.cache.take().unwrap(),
            &mut self.renderer,
        );

        // TODO: Nested redraws!
        let _ = user_interface.update(
            &[core::Event::Window(window::Event::RedrawRequested(
                Instant::now(),
            ))],
            mouse::Cursor::Unavailable,
            &mut self.renderer,
            &mut self.clipboard,
            &mut Vec::new(),
        );

        user_interface.draw(
            &mut self.renderer,
            theme,
            &renderer::Style {
                text_color: style.text_color,
            },
            mouse::Cursor::Unavailable,
        );

        let physical_size = Size::new(
            (self.size.width * scale_factor).round() as u32,
            (self.size.height * scale_factor).round() as u32,
        );

        let rgba = self.renderer.screenshot(
            physical_size,
            scale_factor,
            style.background_color,
        );

        window::Screenshot {
            rgba: Bytes::from(rgba),
            size: physical_size,
            scale_factor,
        }
    }

    /// Turns the [`Emulator`] into its internal state.
    pub fn into_state(self) -> (P::State, core::window::Id) {
        (self.state, self.window)
    }
}

/// The strategy used by an [`Emulator`] when waiting for tasks to finish.
///
/// A [`Mode`] can be used to make an [`Emulator`] wait for side effects to finish before
/// continuing execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Waits for all tasks spawned by an [`Instruction`], as well as all tasks indirectly
    /// spawned by the the results of those tasks.
    ///
    /// This is the default.
    #[default]
    Zen,
    /// Waits only for the tasks directly spawned by an [`Instruction`].
    Patient,
    /// Never waits for any tasks to finish.
    Immediate,
}

impl Mode {
    /// A list of all the available modes.
    pub const ALL: &[Self] = &[Self::Zen, Self::Patient, Self::Immediate];
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Zen => "Zen",
            Self::Patient => "Patient",
            Self::Immediate => "Immediate",
        })
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
