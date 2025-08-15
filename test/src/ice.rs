use crate::Instruction;
use crate::core::Size;
use crate::emulator;
use crate::instruction;

#[derive(Debug, Clone, PartialEq)]
pub struct Ice {
    pub viewport: Size<u32>,
    pub mode: emulator::Mode,
    pub preset: Option<String>,
    pub instructions: Vec<Instruction>,
}

impl Ice {
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
                    mode = Some(match value.trim() {
                        "patient" => emulator::Mode::Patient,
                        "impatient" => emulator::Mode::Impatient,
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
            width = self.viewport.width,
            height = self.viewport.height
        )?;

        writeln!(
            f,
            "mode: {}",
            match self.mode {
                emulator::Mode::Patient => "patient",
                emulator::Mode::Impatient => "impatient",
            }
        )?;

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

#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseError {
    #[error("the ice test has no metadata")]
    NoMetadata,

    #[error("invalid metadata in line {line}: \"{content}\"")]
    InvalidMetadata { line: usize, content: String },

    #[error("invalid viewport in line {line}: \"{value}\"")]
    InvalidViewport { line: usize, value: String },

    #[error("invalid mode in line {line}: \"{value}\"")]
    InvalidMode { line: usize, value: String },

    #[error("unknown metadata field in line {line}: \"{field}\"")]
    UnknownField { line: usize, field: String },

    #[error("metadata is missing the viewport field")]
    MissingViewport,

    #[error("metadata is missing the mode field")]
    MissingMode,

    #[error("invalid instruction in line {line}: {error}")]
    InvalidInstruction {
        line: usize,
        error: instruction::ParseError,
    },
}
