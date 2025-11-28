//! Test your `iced` applications in headless mode.
//!
//! # Basic Usage
//! Let's assume we want to test [the classical counter interface].
//!
//! First, we will want to create a [`Simulator`] of our interface:
//!
//! ```rust,no_run
//! # struct Counter { value: i64 }
//! # impl Counter {
//! #    pub fn view(&self) -> iced_runtime::core::Element<(), iced_runtime::core::Theme, iced_renderer::Renderer> { unimplemented!() }
//! # }
//! use iced_test::simulator;
//!
//! let mut counter = Counter { value: 0 };
//! let mut ui = simulator(counter.view());
//! ```
//!
//! Now we can simulate a user interacting with our interface. Let's use [`Simulator::click`] to click
//! the counter buttons:
//!
//! ```rust,no_run
//! # struct Counter { value: i64 }
//! # impl Counter {
//! #    pub fn view(&self) -> iced_runtime::core::Element<(), iced_runtime::core::Theme, iced_renderer::Renderer> { unimplemented!() }
//! # }
//! # use iced_test::simulator;
//! #
//! # let mut counter = Counter { value: 0 };
//! # let mut ui = simulator(counter.view());
//! #
//! let _ = ui.click("+");
//! let _ = ui.click("+");
//! let _ = ui.click("-");
//! ```
//!
//! [`Simulator::click`] takes a type implementing the [`Selector`] trait. A [`Selector`] describes a way to query the widgets of an interface.
//! In this case, we leverage the [`Selector`] implementation of `&str`, which selects a widget by the text it contains.
//!
//! We can now process any messages produced by these interactions and then assert that the final value of our counter is
//! indeed `1`!
//!
//! ```rust,no_run
//! # struct Counter { value: i64 }
//! # impl Counter {
//! #    pub fn update(&mut self, message: ()) {}
//! #    pub fn view(&self) -> iced_runtime::core::Element<(), iced_runtime::core::Theme, iced_renderer::Renderer> { unimplemented!() }
//! # }
//! # use iced_test::simulator;
//! #
//! # let mut counter = Counter { value: 0 };
//! # let mut ui = simulator(counter.view());
//! #
//! # let _ = ui.click("+");
//! # let _ = ui.click("+");
//! # let _ = ui.click("-");
//! #
//! for message in ui.into_messages() {
//!     counter.update(message);
//! }
//!
//! assert_eq!(counter.value, 1);
//! ```
//!
//! We can even rebuild the interface to make sure the counter _displays_ the proper value with [`Simulator::find`]:
//!
//! ```rust,no_run
//! # struct Counter { value: i64 }
//! # impl Counter {
//! #    pub fn view(&self) -> iced_runtime::core::Element<(), iced_runtime::core::Theme, iced_renderer::Renderer> { unimplemented!() }
//! # }
//! # use iced_test::simulator;
//! #
//! # let mut counter = Counter { value: 0 };
//! let mut ui = simulator(counter.view());
//!
//! assert!(ui.find("1").is_ok(), "Counter should display 1!");
//! ```
//!
//! And that's it! That's the gist of testing `iced` applications!
//!
//! [`Simulator`] contains additional operations you can use to simulate more interactions—like [`tap_key`](Simulator::tap_key) or
//! [`typewrite`](Simulator::typewrite)—and even perform [_snapshot testing_](Simulator::snapshot)!
//!
//! [the classical counter interface]: https://book.iced.rs/architecture.html#dissecting-an-interface
pub use iced_program as program;
pub use iced_renderer as renderer;
pub use iced_runtime as runtime;
pub use iced_runtime::core;

pub use iced_selector as selector;

pub mod emulator;
pub mod ice;
pub mod instruction;
pub mod simulator;

mod error;

pub use emulator::Emulator;
pub use error::Error;
pub use ice::Ice;
pub use instruction::Instruction;
pub use selector::Selector;
pub use simulator::{Simulator, simulator};

use crate::core::Size;
use crate::core::time::{Duration, Instant};
use crate::core::window;

use std::path::Path;

/// Runs an [`Ice`] test suite for the given [`Program`](program::Program).
///
/// Any `.ice` tests will be parsed from the given directory and executed in
/// an [`Emulator`] of the given [`Program`](program::Program).
///
/// Remember that an [`Emulator`] executes the real thing! Side effects _will_
/// take place. It is up to you to ensure your tests have reproducible environments
/// by leveraging [`Preset`][program::Preset].
pub fn run(
    program: impl program::Program + 'static,
    tests_dir: impl AsRef<Path>,
) -> Result<(), Error> {
    use crate::runtime::futures::futures::StreamExt;
    use crate::runtime::futures::futures::channel::mpsc;
    use crate::runtime::futures::futures::executor;

    use std::ffi::OsStr;
    use std::fs;

    let files = fs::read_dir(tests_dir)?;
    let mut tests = Vec::new();

    for file in files {
        let file = file?;

        if file.path().extension().and_then(OsStr::to_str) != Some("ice") {
            continue;
        }

        let content = fs::read_to_string(file.path())?;

        match Ice::parse(&content) {
            Ok(ice) => {
                let preset = if let Some(preset) = &ice.preset {
                    let Some(preset) = program
                        .presets()
                        .iter()
                        .find(|candidate| candidate.name() == preset)
                    else {
                        return Err(Error::PresetNotFound {
                            name: preset.to_owned(),
                            available: program
                                .presets()
                                .iter()
                                .map(program::Preset::name)
                                .map(str::to_owned)
                                .collect(),
                        });
                    };

                    Some(preset)
                } else {
                    None
                };

                tests.push((file, ice, preset));
            }
            Err(error) => {
                return Err(Error::IceParsingFailed {
                    file: file.path().to_path_buf(),
                    error,
                });
            }
        }
    }

    // TODO: Concurrent runtimes
    for (file, ice, preset) in tests {
        let (sender, mut receiver) = mpsc::channel(1);

        let mut emulator = Emulator::with_preset(
            sender,
            &program,
            ice.mode,
            ice.viewport,
            preset,
        );

        let mut instructions = ice.instructions.into_iter();

        loop {
            let event = executor::block_on(receiver.next())
                .expect("emulator runtime should never stop on its own");

            match event {
                emulator::Event::Action(action) => {
                    emulator.perform(&program, action);
                }
                emulator::Event::Failed(instruction) => {
                    return Err(Error::IceTestingFailed {
                        file: file.path().to_path_buf(),
                        instruction,
                    });
                }
                emulator::Event::Ready => {
                    let Some(instruction) = instructions.next() else {
                        break;
                    };

                    emulator.run(&program, instruction);
                }
            }
        }
    }

    Ok(())
}

/// Takes a screenshot of the given [`Program`](program::Program) with the given theme, viewport,
/// and scale factor after running it for the given [`Duration`].
pub fn screenshot<P: program::Program + 'static>(
    program: &P,
    theme: &P::Theme,
    viewport: impl Into<Size>,
    scale_factor: f32,
    duration: Duration,
) -> window::Screenshot {
    use crate::runtime::futures::futures::channel::mpsc;

    let (sender, mut receiver) = mpsc::channel(100);

    let mut emulator = Emulator::new(
        sender,
        program,
        emulator::Mode::Immediate,
        viewport.into(),
    );

    let start = Instant::now();

    loop {
        if let Some(event) = receiver.try_next().ok().flatten() {
            match event {
                emulator::Event::Action(action) => {
                    emulator.perform(program, action);
                }
                emulator::Event::Failed(_) => {
                    unreachable!(
                        "no instructions should be executed during a screenshot"
                    );
                }
                emulator::Event::Ready => {}
            }
        }

        if start.elapsed() >= duration {
            break;
        }

        std::thread::sleep(Duration::from_millis(1));
    }

    emulator.screenshot(program, theme, scale_factor)
}
