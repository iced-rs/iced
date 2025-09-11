use crate::core::keyboard;
use crate::core::mouse;
use crate::core::{Event, Point};
use crate::simulator;

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Interact(Interaction),
    Expect(Expectation),
}

impl Instruction {
    pub fn parse(line: &str) -> Result<Self, ParseError> {
        parser::run(line)
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Interact(interaction) => interaction.fmt(f),
            Instruction::Expect(expectation) => expectation.fmt(f),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Interaction {
    Mouse(Mouse),
    Keyboard(Keyboard),
}

impl Interaction {
    pub fn from_event(event: &Event) -> Option<Self> {
        Some(match event {
            Event::Mouse(mouse) => Self::Mouse(match mouse {
                mouse::Event::CursorMoved { position } => {
                    Mouse::Move(Target::Point(*position))
                }
                mouse::Event::ButtonPressed(button) => Mouse::Press {
                    button: *button,
                    at: None,
                },
                mouse::Event::ButtonReleased(button) => Mouse::Release {
                    button: *button,
                    at: None,
                },
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
                    _ => Keyboard::Typewrite(text.as_ref()?.to_string()),
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
                        && release_at.as_ref().is_none_or(|release_at| {
                            Some(release_at) == press_at.as_ref()
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
                    (
                        Mouse::Press {
                            button,
                            at: Some(press_at),
                        },
                        Mouse::Move(move_at),
                    ) if press_at == move_at => (
                        Self::Mouse(Mouse::Press {
                            button,
                            at: Some(press_at),
                        }),
                        None,
                    ),
                    (
                        Mouse::Click {
                            button,
                            at: Some(click_at),
                        },
                        Mouse::Move(move_at),
                    ) if click_at == move_at => (
                        Self::Mouse(Mouse::Click {
                            button,
                            at: Some(click_at),
                        }),
                        None,
                    ),
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

    pub fn events(
        &self,
        find_target: impl FnOnce(&Target) -> Option<Point>,
    ) -> Option<Vec<Event>> {
        let mouse_move_ =
            |to| Event::Mouse(mouse::Event::CursorMoved { position: to });

        let mouse_press =
            |button| Event::Mouse(mouse::Event::ButtonPressed(button));

        let mouse_release =
            |button| Event::Mouse(mouse::Event::ButtonReleased(button));

        let key_press = |key| simulator::press_key(key, None);

        let key_release = |key| simulator::release_key(key);

        Some(match self {
            Interaction::Mouse(mouse) => match mouse {
                Mouse::Move(to) => vec![mouse_move_(find_target(to)?)],
                Mouse::Press {
                    button,
                    at: Some(at),
                } => vec![mouse_move_(find_target(at)?), mouse_press(*button)],
                Mouse::Press { button, at: None } => {
                    vec![mouse_press(*button)]
                }
                Mouse::Release {
                    button,
                    at: Some(at),
                } => {
                    vec![mouse_move_(find_target(at)?), mouse_release(*button)]
                }
                Mouse::Release { button, at: None } => {
                    vec![mouse_release(*button)]
                }
                Mouse::Click {
                    button,
                    at: Some(at),
                } => {
                    vec![
                        mouse_move_(find_target(at)?),
                        mouse_press(*button),
                        mouse_release(*button),
                    ]
                }
                Mouse::Click { button, at: None } => {
                    vec![mouse_press(*button), mouse_release(*button)]
                }
            },
            Interaction::Keyboard(keyboard) => match keyboard {
                Keyboard::Press(key) => vec![key_press(*key)],
                Keyboard::Release(key) => vec![key_release(*key)],
                Keyboard::Type(key) => vec![key_press(*key), key_release(*key)],
                Keyboard::Typewrite(text) => {
                    simulator::typewrite(text).collect()
                }
            },
        })
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

#[derive(Debug, Clone, PartialEq)]
pub enum Mouse {
    Move(Target),
    Press {
        button: mouse::Button,
        at: Option<Target>,
    },
    Release {
        button: mouse::Button,
        at: Option<Target>,
    },
    Click {
        button: mouse::Button,
        at: Option<Target>,
    },
}

impl fmt::Display for Mouse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mouse::Move(target) => {
                write!(f, "move cursor to {}", target)
            }
            Mouse::Press { button, at } => {
                write!(f, "press {}", format::button_at(*button, at.as_ref()))
            }
            Mouse::Release { button, at } => {
                write!(f, "release {}", format::button_at(*button, at.as_ref()))
            }
            Mouse::Click { button, at } => {
                write!(f, "click {}", format::button_at(*button, at.as_ref()))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    Point(Point),
    Text(String),
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Point(point) => f.write_str(&format::point(*point)),
            Self::Text(text) => f.write_str(&format::string(text)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Keyboard {
    Press(Key),
    Release(Key),
    Type(Key),
    Typewrite(String),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Enter,
    Escape,
    Tab,
    Backspace,
}

impl From<Key> for keyboard::Key {
    fn from(key: Key) -> Self {
        match key {
            Key::Enter => Self::Named(keyboard::key::Named::Enter),
            Key::Escape => Self::Named(keyboard::key::Named::Escape),
            Key::Tab => Self::Named(keyboard::key::Named::Tab),
            Key::Backspace => Self::Named(keyboard::key::Named::Backspace),
        }
    }
}

mod format {
    use super::*;

    pub fn button_at(button: mouse::Button, at: Option<&Target>) -> String {
        let button = self::button(button);

        if let Some(at) = at {
            if button.is_empty() {
                format!("at {}", at)
            } else {
                format!("{} at {}", button, at)
            }
        } else {
            button.to_owned()
        }
    }

    pub fn button(button: mouse::Button) -> &'static str {
        match button {
            mouse::Button::Left => "",
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

    pub fn string(text: &str) -> String {
        format!("\"{}\"", text.escape_default())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expectation {
    Text(String),
}

impl fmt::Display for Expectation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expectation::Text(text) => {
                write!(f, "expect {}", format::string(text))
            }
        }
    }
}

pub use parser::Error as ParseError;

mod parser {
    use super::*;

    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::bytes::{is_not, take_while_m_n};
    use nom::character::complete::{char, multispace0, multispace1};
    use nom::combinator::{
        cut, map, map_opt, map_res, opt, success, value, verify,
    };
    use nom::error::ParseError;
    use nom::multi::fold;
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
        alt((
            map(interaction, Instruction::Interact),
            map(expectation, Instruction::Expect),
        ))
        .parse(input)
    }

    fn interaction(input: &str) -> IResult<&str, Interaction> {
        alt((
            map(mouse, Interaction::Mouse),
            map(keyboard, Interaction::Keyboard),
        ))
        .parse(input)
    }

    fn mouse(input: &str) -> IResult<&str, Mouse> {
        let mouse_move =
            preceded(tag("move cursor to "), target).map(Mouse::Move);

        alt((mouse_move, mouse_click)).parse(input)
    }

    fn mouse_click(input: &str) -> IResult<&str, Mouse> {
        let (input, _) = tag("click ")(input)?;

        let (input, (button, at)) = mouse_button_at(input)?;

        Ok((input, Mouse::Click { button, at }))
    }

    fn mouse_button_at(
        input: &str,
    ) -> IResult<&str, (mouse::Button, Option<Target>)> {
        let (input, button) = mouse_button(input)?;
        let (input, at) = opt(target).parse(input)?;

        Ok((input, (button, at)))
    }

    fn target(input: &str) -> IResult<&str, Target> {
        preceded(
            whitespace(tag("at ")),
            cut(alt((string.map(Target::Text), point.map(Target::Point)))),
        )
        .parse(input)
    }

    fn mouse_button(input: &str) -> IResult<&str, mouse::Button> {
        alt((
            tag("right").map(|_| mouse::Button::Right),
            success(mouse::Button::Left),
        ))
        .parse(input)
    }

    fn keyboard(input: &str) -> IResult<&str, Keyboard> {
        alt((
            map(preceded(tag("type "), string), Keyboard::Typewrite),
            map(preceded(tag("type "), key), Keyboard::Type),
        ))
        .parse(input)
    }

    fn expectation(input: &str) -> IResult<&str, Expectation> {
        map(preceded(tag("expect "), string), |text| {
            Expectation::Text(text)
        })
        .parse(input)
    }

    fn key(input: &str) -> IResult<&str, Key> {
        alt((
            map(tag("enter"), |_| Key::Enter),
            map(tag("escape"), |_| Key::Escape),
            map(tag("tab"), |_| Key::Tab),
            map(tag("backspace"), |_| Key::Backspace),
        ))
        .parse(input)
    }

    fn point(input: &str) -> IResult<&str, Point> {
        let comma = whitespace(char(','));

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

    pub fn whitespace<'a, O, E: ParseError<&'a str>, F>(
        inner: F,
    ) -> impl Parser<&'a str, Output = O, Error = E>
    where
        F: Parser<&'a str, Output = O, Error = E>,
    {
        delimited(multispace0, inner, multispace0)
    }

    // Taken from https://github.com/rust-bakery/nom/blob/51c3c4e44fa78a8a09b413419372b97b2cc2a787/examples/string.rs
    //
    // Copyright (c) 2014-2019 Geoffroy Couprie
    //
    // Permission is hereby granted, free of charge, to any person obtaining
    // a copy of this software and associated documentation files (the
    // "Software"), to deal in the Software without restriction, including
    // without limitation the rights to use, copy, modify, merge, publish,
    // distribute, sublicense, and/or sell copies of the Software, and to
    // permit persons to whom the Software is furnished to do so, subject to
    // the following conditions:
    //
    // The above copyright notice and this permission notice shall be
    // included in all copies or substantial portions of the Software.
    //
    // THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
    // EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
    // MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
    // NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
    // LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
    // OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
    // WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
    fn string(input: &str) -> IResult<&str, String> {
        #[derive(Debug, Clone, Copy)]
        enum Fragment<'a> {
            Literal(&'a str),
            EscapedChar(char),
            EscapedWS,
        }

        fn fragment(input: &str) -> IResult<&str, Fragment<'_>> {
            alt((
                map(string_literal, Fragment::Literal),
                map(escaped_char, Fragment::EscapedChar),
                value(Fragment::EscapedWS, escaped_whitespace),
            ))
            .parse(input)
        }

        fn string_literal<'a, E: ParseError<&'a str>>(
            input: &'a str,
        ) -> IResult<&'a str, &'a str, E> {
            let not_quote_slash = is_not("\"\\");

            verify(not_quote_slash, |s: &str| !s.is_empty()).parse(input)
        }

        fn unicode(input: &str) -> IResult<&str, char> {
            let parse_hex =
                take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit());

            let parse_delimited_hex =
                preceded(char('u'), delimited(char('{'), parse_hex, char('}')));

            let parse_u32 = map_res(parse_delimited_hex, move |hex| {
                u32::from_str_radix(hex, 16)
            });

            map_opt(parse_u32, std::char::from_u32).parse(input)
        }

        fn escaped_char(input: &str) -> IResult<&str, char> {
            preceded(
                char('\\'),
                alt((
                    unicode,
                    value('\n', char('n')),
                    value('\r', char('r')),
                    value('\t', char('t')),
                    value('\u{08}', char('b')),
                    value('\u{0C}', char('f')),
                    value('\\', char('\\')),
                    value('/', char('/')),
                    value('"', char('"')),
                )),
            )
            .parse(input)
        }

        fn escaped_whitespace(input: &str) -> IResult<&str, &str> {
            preceded(char('\\'), multispace1).parse(input)
        }

        let build_string =
            fold(0.., fragment, String::new, |mut string, fragment| {
                match fragment {
                    Fragment::Literal(s) => string.push_str(s),
                    Fragment::EscapedChar(c) => string.push(c),
                    Fragment::EscapedWS => {}
                }
                string
            });

        delimited(char('"'), build_string, char('"')).parse(input)
    }
}
