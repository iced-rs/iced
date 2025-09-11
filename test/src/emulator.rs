use crate::core;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget;
use crate::core::{Element, Point, Size};
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
use crate::runtime::window;
use crate::runtime::{Task, UserInterface};
use crate::selector;
use crate::{Instruction, Selector};

use std::fmt;

#[allow(missing_debug_implementations)]
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

#[allow(missing_debug_implementations)]
pub enum Event<P: Program> {
    Action(Action<P>),
    Failed(Instruction),
    Ready,
}

#[allow(missing_debug_implementations)]
pub enum Action<P: Program> {
    Runtime(runtime::Action<P::Message>),
    CountDown,
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
                        stream.map(Action::Runtime).map(Event::Action).boxed(),
                    );
                }
            }
        }
    }

    pub fn perform(&mut self, program: &P, action: Action<P>) {
        match action {
            Action::CountDown => {
                if self.pending_tasks > 0 {
                    self.pending_tasks -= 1;

                    if self.pending_tasks == 0 {
                        self.runtime.send(Event::Ready);
                    }
                }
            }
            Action::Runtime(action) => match action {
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
                runtime::Action::Window(action) => match action {
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
                            let _ = sender.send(core::window::Mode::Windowed);
                        }
                    }
                    _ => {
                        // Ignored
                    }
                },
                runtime::Action::System(action) => {
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
                        use selector::target::Bounded;
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

    pub fn wait_for(&mut self, task: Task<P::Message>) {
        if let Some(stream) = task::into_stream(task) {
            match self.mode {
                Mode::Zen => {
                    self.pending_tasks += 1;

                    self.runtime.run(
                        stream
                            .map(Action::Runtime)
                            .map(Event::Action)
                            .chain(stream::once(async {
                                Event::Action(Action::CountDown)
                            }))
                            .boxed(),
                    );
                }
                Mode::Patient => {
                    self.runtime.run(
                        stream
                            .map(Action::Runtime)
                            .map(Event::Action)
                            .chain(stream::once(async { Event::Ready }))
                            .boxed(),
                    );
                }
                Mode::Impatient => {
                    self.runtime.run(
                        stream.map(Action::Runtime).map(Event::Action).boxed(),
                    );
                    self.runtime.send(Event::Ready);
                }
            }
        } else if self.pending_tasks == 0 {
            self.runtime.send(Event::Ready);
        }
    }

    pub fn resubscribe(&mut self, program: &P) {
        self.runtime
            .track(subscription::into_recipes(self.runtime.enter(|| {
                program.subscription(&self.state).map(|message| {
                    Event::Action(Action::Runtime(runtime::Action::Output(
                        message,
                    )))
                })
            })));
    }

    pub fn view(
        &self,
        program: &P,
    ) -> Element<'_, P::Message, P::Theme, P::Renderer> {
        program.view(&self.state, self.window)
    }

    pub fn theme(&self, program: &P) -> Option<P::Theme> {
        program.theme(&self.state, self.window)
    }

    pub fn into_state(self) -> (P::State, core::window::Id) {
        (self.state, self.window)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Zen,
    Patient,
    Impatient,
}

impl Mode {
    pub const ALL: &[Self] = &[Self::Zen, Self::Patient, Self::Impatient];
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Zen => "Zen",
            Self::Patient => "Patient",
            Self::Impatient => "Impatient",
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
