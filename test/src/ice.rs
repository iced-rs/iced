//! A shareable, simple format of end-to-end tests.
use crate::Instruction;
use crate::core::Size;
use crate::emulator;
use crate::instruction;

/// An end-to-end test for iced applications.
///
/// Ice tests encode a certain configuration together with a sequence of instructions.
/// An ice test passes if all the instructions can be executed successfully.
///
/// Normally, ice tests are run by an [`Emulator`](crate::Emulator) in continuous
/// integration pipelines.
///
/// Ice tests can be easily run by saving them as `.ice` files in a folder and simply
/// calling [`run`](crate::run). These test files can be recorded by enabling the `tester`
/// feature flag in the root crate.
#[derive(Debug, Clone, PartialEq)]
pub struct Ice {
    /// The viewport [`Size`] that must be used for the test.
    pub viewport: Size,
    /// The [`emulator::Mode`] that must be used for the test.
    pub mode: emulator::Mode,
    /// The name of the [`Preset`](crate::program::Preset) that must be used for the test.
    pub preset: Option<String>,
    /// The sequence of instructions of the test.
    pub instructions: Vec<Instruction>,
}

impl Ice {
    /// Parses an [`Ice`] test from its textual representation.
    ///
    /// Here is an example of the [`Ice`] test syntax:
    ///
    /// ```text
    /// viewport: 500x800
    /// mode: Immediate
    /// preset: Empty
    /// -----
    /// click "What needs to be done?"
    /// type "Create the universe"
    /// type enter
    /// type "Make an apple pie"
    /// type enter
    /// expect "2 tasks left"
    /// click "Create the universe"
    /// expect "1 task left"
    /// click "Make an apple pie"
    /// expect "0 tasks left"
    /// ```
    ///
    /// This syntax is _very_ experimental and extremely likely to change often.
    /// For this reason, it is reserved for advanced users that want to early test it.
    ///
    /// Currently, in order to use it, you will need to earn the right and prove you understand
    /// its experimental nature by reading the code!
    pub fn parse(content: &str) -> Result<Self, ParseError> {
        let Some((metadata, rest)) = content.split_once("-") else {
            return Err(ParseError::NoMetadata);
        };

        let mut viewport = None;
        let mut mode = None;
        let mut preset = None;

        for (i, line) in metadata.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            let Some((field, value)) = line.split_once(':') else {
                return Err(ParseError::InvalidMetadata {
                    line: i,
                    content: line.to_owned(),
                });
            };

            match field.trim() {
                "viewport" => {
                    viewport = Some(
                        if let Some((width, height)) =
                            value.trim().split_once('x')
                            && let Ok(width) = width.parse()
                            && let Ok(height) = height.parse()
                        {
                            Size::new(width, height)
                        } else {
                            return Err(ParseError::InvalidViewport {
                                line: i,
                                value: value.to_owned(),
                            });
                        },
                    );
                }
                "mode" => {
                    mode = Some(match value.trim().to_lowercase().as_str() {
                        "zen" => emulator::Mode::Zen,
                        "patient" => emulator::Mode::Patient,
                        "immediate" => emulator::Mode::Immediate,
                        _ => {
                            return Err(ParseError::InvalidMode {
                                line: i,
                                value: value.to_owned(),
                            });
                        }
                    });
                }
                "preset" => {
                    preset = Some(value.trim().to_owned());
                }
                field => {
                    return Err(ParseError::UnknownField {
                        line: i,
                        field: field.to_owned(),
                    });
                }
            }
        }

        let Some(viewport) = viewport else {
            return Err(ParseError::MissingViewport);
        };

        let Some(mode) = mode else {
            return Err(ParseError::MissingMode);
        };

        let instructions = rest
            .lines()
            .skip(1)
            .enumerate()
            .map(|(i, line)| {
                Instruction::parse(line).map_err(|error| {
                    ParseError::InvalidInstruction {
                        line: metadata.lines().count() + 1 + i,
                        error,
                    }
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            viewport,
            mode,
            preset,
            instructions,
        })
    }
}

impl std::fmt::Display for Ice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "viewport: {width}x{height}",
            width = self.viewport.width as u32, // TODO
            height = self.viewport.height as u32, // TODO
        )?;

        writeln!(f, "mode: {}", self.mode)?;

        if let Some(preset) = &self.preset {
            writeln!(f, "preset: {preset}")?;
        }

        f.write_str("-----\n")?;

        for instruction in &self.instructions {
            instruction.fmt(f)?;
            f.write_str("\n")?;
        }

        Ok(())
    }
}

/// An error produced during [`Ice::parse`].
#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseError {
    /// No metadata is present.
    #[error("the ice test has no metadata")]
    NoMetadata,

    /// The metadata is invalid.
    #[error("invalid metadata in line {line}: \"{content}\"")]
    InvalidMetadata {
        /// The number of the invalid line.
        line: usize,
        /// The content of the invalid line.
        content: String,
    },

    /// The viewport is invalid.
    #[error("invalid viewport in line {line}: \"{value}\"")]
    InvalidViewport {
        /// The number of the invalid line.
        line: usize,

        /// The invalid value.
        value: String,
    },

    /// The [`emulator::Mode`] is invalid.
    #[error("invalid mode in line {line}: \"{value}\"")]
    InvalidMode {
        /// The number of the invalid line.
        line: usize,
        /// The invalid value.
        value: String,
    },

    /// A metadata field is unknown.
    #[error("unknown metadata field in line {line}: \"{field}\"")]
    UnknownField {
        /// The number of the invalid line.
        line: usize,
        /// The name of the unknown field.
        field: String,
    },

    /// Viewport metadata is missing.
    #[error("metadata is missing the viewport field")]
    MissingViewport,

    /// [`emulator::Mode`] metadata is missing.
    #[error("metadata is missing the mode field")]
    MissingMode,

    /// An [`Instruction`] failed to parse.
    #[error("invalid instruction in line {line}: {error}")]
    InvalidInstruction {
        /// The number of the invalid line.
        line: usize,
        /// The parse error.
        error: instruction::ParseError,
    },
}
