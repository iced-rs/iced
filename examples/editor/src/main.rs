use iced::executor;
use iced::theme::{self, Theme};
use iced::widget::{
    button, column, container, horizontal_space, pick_list, row, text,
    text_editor, tooltip,
};
use iced::{Application, Command, Element, Font, Length, Settings};

use highlighter::Highlighter;

use std::ffi;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub fn main() -> iced::Result {
    Editor::run(Settings {
        fonts: vec![include_bytes!("../fonts/icons.ttf").as_slice().into()],
        default_font: Font {
            monospaced: true,
            ..Font::with_name("Hasklug Nerd Font Mono")
        },
        ..Settings::default()
    })
}

struct Editor {
    file: Option<PathBuf>,
    content: text_editor::Content,
    theme: highlighter::Theme,
    is_loading: bool,
    is_dirty: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    ThemeSelected(highlighter::Theme),
    NewFile,
    OpenFile,
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    SaveFile,
    FileSaved(Result<PathBuf, Error>),
}

impl Application for Editor {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                file: None,
                content: text_editor::Content::new(),
                theme: highlighter::Theme::SolarizedDark,
                is_loading: true,
                is_dirty: false,
            },
            Command::perform(load_file(default_file()), Message::FileOpened),
        )
    }

    fn title(&self) -> String {
        String::from("Editor - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Edit(action) => {
                self.is_dirty = self.is_dirty || action.is_edit();

                self.content.edit(action);

                Command::none()
            }
            Message::ThemeSelected(theme) => {
                self.theme = theme;

                Command::none()
            }
            Message::NewFile => {
                if !self.is_loading {
                    self.file = None;
                    self.content = text_editor::Content::new();
                }

                Command::none()
            }
            Message::OpenFile => {
                if self.is_loading {
                    Command::none()
                } else {
                    self.is_loading = true;

                    Command::perform(open_file(), Message::FileOpened)
                }
            }
            Message::FileOpened(result) => {
                self.is_loading = false;
                self.is_dirty = false;

                if let Ok((path, contents)) = result {
                    self.file = Some(path);
                    self.content = text_editor::Content::with(&contents);
                }

                Command::none()
            }
            Message::SaveFile => {
                if self.is_loading {
                    Command::none()
                } else {
                    self.is_loading = true;

                    let mut contents = self.content.lines().enumerate().fold(
                        String::new(),
                        |mut contents, (i, line)| {
                            if i > 0 {
                                contents.push('\n');
                            }

                            contents.push_str(&line);

                            contents
                        },
                    );

                    if !contents.ends_with('\n') {
                        contents.push('\n');
                    }

                    Command::perform(
                        save_file(self.file.clone(), contents),
                        Message::FileSaved,
                    )
                }
            }
            Message::FileSaved(result) => {
                self.is_loading = false;

                if let Ok(path) = result {
                    self.file = Some(path);
                    self.is_dirty = false;
                }

                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let controls = row![
            action(new_icon(), "New file", Some(Message::NewFile)),
            action(
                open_icon(),
                "Open file",
                (!self.is_loading).then_some(Message::OpenFile)
            ),
            action(
                save_icon(),
                "Save file",
                self.is_dirty.then_some(Message::SaveFile)
            ),
            horizontal_space(Length::Fill),
            pick_list(
                highlighter::Theme::ALL,
                Some(self.theme),
                Message::ThemeSelected
            )
            .text_size(14)
            .padding([5, 10])
        ]
        .spacing(10);

        let status = row![
            text(if let Some(path) = &self.file {
                let path = path.display().to_string();

                if path.len() > 60 {
                    format!("...{}", &path[path.len() - 40..])
                } else {
                    path
                }
            } else {
                String::from("New file")
            }),
            horizontal_space(Length::Fill),
            text({
                let (line, column) = self.content.cursor_position();

                format!("{}:{}", line + 1, column + 1)
            })
        ]
        .spacing(10);

        column![
            controls,
            text_editor(&self.content)
                .on_edit(Message::Edit)
                .highlight::<Highlighter>(highlighter::Settings {
                    theme: self.theme,
                    extension: self
                        .file
                        .as_deref()
                        .and_then(Path::extension)
                        .and_then(ffi::OsStr::to_str)
                        .map(str::to_string)
                        .unwrap_or(String::from("rs")),
                }),
            status,
        ]
        .spacing(10)
        .padding(10)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    DialogClosed,
    IoError(io::ErrorKind),
}

fn default_file() -> PathBuf {
    PathBuf::from(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")))
}

async fn open_file() -> Result<(PathBuf, Arc<String>), Error> {
    let picked_file = rfd::AsyncFileDialog::new()
        .set_title("Open a text file...")
        .pick_file()
        .await
        .ok_or(Error::DialogClosed)?;

    load_file(picked_file.path().to_owned()).await
}

async fn load_file(path: PathBuf) -> Result<(PathBuf, Arc<String>), Error> {
    let contents = tokio::fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .map_err(|error| Error::IoError(error.kind()))?;

    Ok((path, contents))
}

async fn save_file(
    path: Option<PathBuf>,
    contents: String,
) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        path
    } else {
        rfd::AsyncFileDialog::new()
            .save_file()
            .await
            .as_ref()
            .map(rfd::FileHandle::path)
            .map(Path::to_owned)
            .ok_or(Error::DialogClosed)?
    };

    tokio::fs::write(&path, contents)
        .await
        .map_err(|error| Error::IoError(error.kind()))?;

    Ok(path)
}

fn action<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    label: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let action =
        button(container(content).width(Length::Fill).center_x()).width(40);

    if let Some(on_press) = on_press {
        tooltip(
            action.on_press(on_press),
            label,
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box)
        .into()
    } else {
        action.style(theme::Button::Secondary).into()
    }
}

fn new_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e800}')
}

fn save_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e801}')
}

fn open_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0f115}')
}

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("editor-icons");

    text(codepoint).font(ICON_FONT).into()
}

mod highlighter {
    use iced::advanced::text::highlighter;
    use iced::widget::text_editor;
    use iced::{Color, Font};

    use std::ops::Range;
    use syntect::highlighting;
    use syntect::parsing::{self, SyntaxReference};

    #[derive(Debug, Clone, PartialEq)]
    pub struct Settings {
        pub theme: Theme,
        pub extension: String,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Theme {
        SolarizedDark,
        InspiredGitHub,
        Base16Mocha,
    }

    impl Theme {
        pub const ALL: &[Self] =
            &[Self::SolarizedDark, Self::InspiredGitHub, Self::Base16Mocha];

        fn key(&self) -> &'static str {
            match self {
                Theme::InspiredGitHub => "InspiredGitHub",
                Theme::Base16Mocha => "base16-mocha.dark",
                Theme::SolarizedDark => "Solarized (dark)",
            }
        }
    }

    impl std::fmt::Display for Theme {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Theme::InspiredGitHub => write!(f, "Inspired GitHub"),
                Theme::Base16Mocha => write!(f, "Mocha"),
                Theme::SolarizedDark => write!(f, "Solarized Dark"),
            }
        }
    }

    pub struct Highlight(highlighting::StyleModifier);

    impl text_editor::Highlight for Highlight {
        fn format(&self, _theme: &iced::Theme) -> highlighter::Format<Font> {
            highlighter::Format {
                color: self.0.foreground.map(|color| {
                    Color::from_rgba8(
                        color.r,
                        color.g,
                        color.b,
                        color.a as f32 / 255.0,
                    )
                }),
                font: None,
            }
        }
    }

    pub struct Highlighter {
        syntaxes: parsing::SyntaxSet,
        syntax: SyntaxReference,
        theme: highlighting::Theme,
        caches: Vec<(parsing::ParseState, parsing::ScopeStack)>,
        current_line: usize,
    }

    const LINES_PER_SNAPSHOT: usize = 50;

    impl highlighter::Highlighter for Highlighter {
        type Settings = Settings;
        type Highlight = Highlight;

        type Iterator<'a> =
            Box<dyn Iterator<Item = (Range<usize>, Self::Highlight)> + 'a>;

        fn new(settings: &Self::Settings) -> Self {
            let syntaxes = parsing::SyntaxSet::load_defaults_nonewlines();

            let syntax = syntaxes
                .find_syntax_by_token(&settings.extension)
                .unwrap_or_else(|| syntaxes.find_syntax_plain_text());

            let theme = highlighting::ThemeSet::load_defaults()
                .themes
                .remove(settings.theme.key())
                .unwrap();

            let parser = parsing::ParseState::new(syntax);
            let stack = parsing::ScopeStack::new();

            Highlighter {
                syntax: syntax.clone(),
                syntaxes,
                theme,
                caches: vec![(parser, stack)],
                current_line: 0,
            }
        }

        fn update(&mut self, new_settings: &Self::Settings) {
            self.syntax = self
                .syntaxes
                .find_syntax_by_token(&new_settings.extension)
                .unwrap_or_else(|| self.syntaxes.find_syntax_plain_text())
                .clone();

            self.theme = highlighting::ThemeSet::load_defaults()
                .themes
                .remove(new_settings.theme.key())
                .unwrap();

            // Restart the highlighter
            self.change_line(0);
        }

        fn change_line(&mut self, line: usize) {
            let snapshot = line / LINES_PER_SNAPSHOT;

            if snapshot <= self.caches.len() {
                self.caches.truncate(snapshot);
                self.current_line = snapshot * LINES_PER_SNAPSHOT;
            } else {
                self.caches.truncate(1);
                self.current_line = 0;
            }

            let (parser, stack) =
                self.caches.last().cloned().unwrap_or_else(|| {
                    (
                        parsing::ParseState::new(&self.syntax),
                        parsing::ScopeStack::new(),
                    )
                });

            self.caches.push((parser, stack));
        }

        fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
            if self.current_line / LINES_PER_SNAPSHOT >= self.caches.len() {
                let (parser, stack) =
                    self.caches.last().expect("Caches must not be empty");

                self.caches.push((parser.clone(), stack.clone()));
            }

            self.current_line += 1;

            let (parser, stack) =
                self.caches.last_mut().expect("Caches must not be empty");

            let ops =
                parser.parse_line(line, &self.syntaxes).unwrap_or_default();

            let highlighter = highlighting::Highlighter::new(&self.theme);

            Box::new(
                ScopeRangeIterator {
                    ops,
                    line_length: line.len(),
                    index: 0,
                    last_str_index: 0,
                }
                .filter_map(move |(range, scope)| {
                    let _ = stack.apply(&scope);

                    if range.is_empty() {
                        None
                    } else {
                        Some((
                            range,
                            Highlight(
                                highlighter.style_mod_for_stack(&stack.scopes),
                            ),
                        ))
                    }
                }),
            )
        }

        fn current_line(&self) -> usize {
            self.current_line
        }
    }

    pub struct ScopeRangeIterator {
        ops: Vec<(usize, parsing::ScopeStackOp)>,
        line_length: usize,
        index: usize,
        last_str_index: usize,
    }

    impl Iterator for ScopeRangeIterator {
        type Item = (std::ops::Range<usize>, parsing::ScopeStackOp);

        fn next(&mut self) -> Option<Self::Item> {
            if self.index > self.ops.len() {
                return None;
            }

            let next_str_i = if self.index == self.ops.len() {
                self.line_length
            } else {
                self.ops[self.index].0
            };

            let range = self.last_str_index..next_str_i;
            self.last_str_index = next_str_i;

            let op = if self.index == 0 {
                parsing::ScopeStackOp::Noop
            } else {
                self.ops[self.index - 1].1.clone()
            };

            self.index += 1;
            Some((range, op))
        }
    }
}
