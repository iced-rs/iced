use crate::core::keyboard;
use crate::core::mouse;
use crate::core::{Event, Point};

use std::fmt;

#[derive(Debug, Clone)]
pub enum Instruction {
    Interact(Interaction),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Interact(interaction) => interaction.fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Interaction {
    Mouse(Mouse),
    Keyboard(Keyboard),
}

impl Interaction {
    pub fn from_event(event: Event) -> Option<Self> {
        Some(match event {
            Event::Mouse(mouse) => Self::Mouse(match mouse {
                mouse::Event::CursorMoved { position } => Mouse::Move(position),
                mouse::Event::ButtonPressed(button) => {
                    Mouse::Press { button, at: None }
                }
                mouse::Event::ButtonReleased(button) => {
                    Mouse::Release { button, at: None }
                }
                _ => None?,
            }),
            Event::Keyboard(keyboard) => Self::Keyboard(match keyboard {
                keyboard::Event::KeyPressed { key, text, .. } => match key {
                    keyboard::Key::Named(keyboard::key::Named::Enter) => {
                        Keyboard::Press(Key::Enter)
                    }
                    keyboard::Key::Named(keyboard::key::Named::Escape) => {
                        Keyboard::Press(Key::Escape)
                    }
                    keyboard::Key::Named(keyboard::key::Named::Tab) => {
                        Keyboard::Press(Key::Tab)
                    }
                    keyboard::Key::Named(keyboard::key::Named::Backspace) => {
                        Keyboard::Press(Key::Backspace)
                    }
                    _ => Keyboard::Typewrite(text?.to_string()),
                },
                keyboard::Event::KeyReleased { key, .. } => match key {
                    keyboard::Key::Named(keyboard::key::Named::Enter) => {
                        Keyboard::Release(Key::Enter)
                    }
                    keyboard::Key::Named(keyboard::key::Named::Escape) => {
                        Keyboard::Release(Key::Escape)
                    }
                    keyboard::Key::Named(keyboard::key::Named::Tab) => {
                        Keyboard::Release(Key::Tab)
                    }
                    keyboard::Key::Named(keyboard::key::Named::Backspace) => {
                        Keyboard::Release(Key::Backspace)
                    }
                    _ => None?,
                },
                keyboard::Event::ModifiersChanged(_) => None?,
            }),
            _ => None?,
        })
    }

    pub fn merge(self, next: Self) -> (Self, Option<Self>) {
        match (self, next) {
            (Self::Mouse(current), Self::Mouse(next)) => {
                match (current, next) {
                    (Mouse::Move(_), Mouse::Move(to)) => {
                        (Self::Mouse(Mouse::Move(to)), None)
                    }
                    (Mouse::Move(to), Mouse::Press { button, at: None }) => (
                        Self::Mouse(Mouse::Press {
                            button,
                            at: Some(to),
                        }),
                        None,
                    ),
                    (Mouse::Move(to), Mouse::Release { button, at: None }) => (
                        Self::Mouse(Mouse::Release {
                            button,
                            at: Some(to),
                        }),
                        None,
                    ),
                    (
                        Mouse::Press {
                            button: press,
                            at: press_at,
                        },
                        Mouse::Release {
                            button: release,
                            at: release_at,
                        },
                    ) if press == release
                        && release_at.is_none_or(|release_at| {
                            Some(release_at) == press_at
                        }) =>
                    {
                        (
                            Self::Mouse(Mouse::Click {
                                button: press,
                                at: press_at,
                            }),
                            None,
                        )
                    }
                    (current, next) => {
                        (Self::Mouse(current), Some(Self::Mouse(next)))
                    }
                }
            }
            (Self::Keyboard(current), Self::Keyboard(next)) => {
                match (current, next) {
                    (
                        Keyboard::Typewrite(current),
                        Keyboard::Typewrite(next),
                    ) => (
                        Self::Keyboard(Keyboard::Typewrite(format!(
                            "{current}{next}"
                        ))),
                        None,
                    ),
                    (Keyboard::Press(current), Keyboard::Release(next))
                        if current == next =>
                    {
                        (Self::Keyboard(Keyboard::Type(current)), None)
                    }
                    (current, next) => {
                        (Self::Keyboard(current), Some(Self::Keyboard(next)))
                    }
                }
            }
            (current, next) => (current, Some(next)),
        }
    }
}

impl fmt::Display for Interaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Interaction::Mouse(mouse) => mouse.fmt(f),
            Interaction::Keyboard(keyboard) => keyboard.fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Mouse {
    Move(Point),
    Press {
        button: mouse::Button,
        at: Option<Point>,
    },
    Release {
        button: mouse::Button,
        at: Option<Point>,
    },
    Click {
        button: mouse::Button,
        at: Option<Point>,
    },
}

impl fmt::Display for Mouse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mouse::Move(point) => {
                write!(f, "move cursor to ({:.2}, {:.2})", point.x, point.y)
            }
            Mouse::Press { button, at } => {
                write!(f, "press {}", format::button_at(*button, *at))
            }
            Mouse::Release { button, at } => {
                write!(f, "release {}", format::button_at(*button, *at))
            }
            Mouse::Click { button, at } => {
                write!(f, "click {}", format::button_at(*button, *at))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Keyboard {
    Press(Key),
    Release(Key),
    Type(Key),
    Typewrite(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Enter,
    Escape,
    Tab,
    Backspace,
}

impl fmt::Display for Keyboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Keyboard::Press(key) => {
                write!(f, "press {}", format::key(*key))
            }
            Keyboard::Release(key) => {
                write!(f, "release {}", format::key(*key))
            }
            Keyboard::Type(key) => {
                write!(f, "type {}", format::key(*key))
            }
            Keyboard::Typewrite(text) => {
                write!(f, "type \"{text}\"")
            }
        }
    }
}

mod format {
    use super::*;

    pub fn button_at(button: mouse::Button, at: Option<Point>) -> String {
        if let Some(at) = at {
            format!("{} at {}", self::button(button), point(at))
        } else {
            self::button(button).to_owned()
        }
    }

    pub fn button(button: mouse::Button) -> &'static str {
        match button {
            mouse::Button::Left => "left",
            mouse::Button::Right => "right",
            mouse::Button::Middle => "middle",
            mouse::Button::Back => "back",
            mouse::Button::Forward => "forward",
            mouse::Button::Other(_) => "other",
        }
    }

    pub fn point(point: Point) -> String {
        format!("({:.2}, {:.2})", point.x, point.y)
    }

    pub fn key(key: Key) -> &'static str {
        match key {
            Key::Enter => "enter",
            Key::Escape => "escape",
            Key::Tab => "tab",
            Key::Backspace => "backspace",
        }
    }
}

pub use parser::{Error as ParseError, run as parse};

mod parser {
    use super::*;

    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::character::complete::{char, multispace0};
    use nom::combinator::{map, opt};
    use nom::number::float;
    use nom::sequence::{delimited, preceded, separated_pair};
    use nom::{Finish, IResult, Parser};

    #[derive(Debug, Clone, thiserror::Error)]
    #[error("parse error: {0}")]
    pub struct Error(nom::error::Error<String>);

    pub fn run(input: &str) -> Result<Instruction, Error> {
        match instruction.parse_complete(input).finish() {
            Ok((_rest, instruction)) => Ok(instruction),
            Err(error) => Err(Error(error.cloned())),
        }
    }

    fn instruction(input: &str) -> IResult<&str, Instruction> {
        map(interaction, Instruction::Interact).parse(input)
    }

    fn interaction(input: &str) -> IResult<&str, Interaction> {
        map(mouse, Interaction::Mouse).parse(input)
    }

    fn mouse(input: &str) -> IResult<&str, Mouse> {
        let mouse_move =
            preceded(tag("move cursor to "), point).map(Mouse::Move);

        alt((mouse_move, mouse_click)).parse(input)
    }

    fn mouse_click(input: &str) -> IResult<&str, Mouse> {
        let (input, _) = tag("click ")(input)?;

        let (input, (button, at)) = mouse_button_at(input)?;

        Ok((input, Mouse::Click { button, at }))
    }

    fn mouse_button_at(
        input: &str,
    ) -> IResult<&str, (mouse::Button, Option<Point>)> {
        let (input, button) = mouse_button(input)?;
        let (input, at) = opt(preceded(tag(" at "), point)).parse(input)?;

        Ok((input, (button, at)))
    }

    fn mouse_button(input: &str) -> IResult<&str, mouse::Button> {
        alt((
            tag("left").map(|_| mouse::Button::Left),
            tag("right").map(|_| mouse::Button::Right),
        ))
        .parse(input)
    }

    fn point(input: &str) -> IResult<&str, Point> {
        let comma = (multispace0, char(','), multispace0);

        map(
            delimited(
                char('('),
                separated_pair(float(), comma, float()),
                char(')'),
            ),
            |(x, y)| Point { x, y },
        )
        .parse(input)
    }
}
