use iced::widget::{column, horizontal_space, pick_list, row, text_editor};
use iced::{Element, Font, Length, Sandbox, Settings, Theme};

use highlighter::Highlighter;

pub fn main() -> iced::Result {
    Editor::run(Settings::default())
}

struct Editor {
    content: text_editor::Content,
    theme: highlighter::Theme,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    ThemeSelected(highlighter::Theme),
}

impl Sandbox for Editor {
    type Message = Message;

    fn new() -> Self {
        Self {
            content: text_editor::Content::with(include_str!("main.rs")),
            theme: highlighter::Theme::SolarizedDark,
        }
    }

    fn title(&self) -> String {
        String::from("Editor - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Edit(action) => {
                self.content.edit(action);
            }
            Message::ThemeSelected(theme) => {
                self.theme = theme;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        column![
            row![
                horizontal_space(Length::Fill),
                pick_list(
                    highlighter::Theme::ALL,
                    Some(self.theme),
                    Message::ThemeSelected
                )
                .padding([5, 10])
            ]
            .spacing(10),
            text_editor(&self.content)
                .on_edit(Message::Edit)
                .font(Font::with_name("Hasklug Nerd Font Mono"))
                .highlight::<Highlighter>(highlighter::Settings {
                    theme: self.theme,
                    extension: String::from("rs"),
                }),
        ]
        .spacing(10)
        .padding(20)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
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
