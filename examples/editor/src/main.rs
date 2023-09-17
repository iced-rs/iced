use iced::widget::{container, text_editor};
use iced::{Element, Font, Sandbox, Settings, Theme};

use highlighter::Highlighter;

pub fn main() -> iced::Result {
    Editor::run(Settings::default())
}

struct Editor {
    content: text_editor::Content,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
}

impl Sandbox for Editor {
    type Message = Message;

    fn new() -> Self {
        Self {
            content: text_editor::Content::with(include_str!(
                "../../../README.md"
            )),
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
        }
    }

    fn view(&self) -> Element<Message> {
        container(
            text_editor(&self.content)
                .on_edit(Message::Edit)
                .font(Font::with_name("Hasklug Nerd Font Mono"))
                .highlight::<Highlighter>(highlighter::Settings {
                    token: String::from("md"),
                }),
        )
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
    use iced::{Color, Font, Theme};

    use std::ops::Range;
    use syntect::highlighting;
    use syntect::parsing;

    #[derive(Debug, Clone, Hash)]
    pub struct Settings {
        pub token: String,
    }

    pub struct Highlight(highlighting::StyleModifier);

    impl text_editor::Highlight for Highlight {
        fn format(&self, _theme: &Theme) -> highlighter::Format<Font> {
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
        parser: parsing::ParseState,
        stack: parsing::ScopeStack,
        theme: highlighting::Theme,
        token: String,
        current_line: usize,
    }

    impl highlighter::Highlighter for Highlighter {
        type Settings = Settings;
        type Highlight = Highlight;

        type Iterator<'a> =
            Box<dyn Iterator<Item = (Range<usize>, Self::Highlight)> + 'a>;

        fn new(settings: &Self::Settings) -> Self {
            let syntaxes = parsing::SyntaxSet::load_defaults_nonewlines();

            let syntax = syntaxes
                .find_syntax_by_token(&settings.token)
                .unwrap_or_else(|| syntaxes.find_syntax_plain_text());

            let parser = parsing::ParseState::new(&syntax);
            let stack = parsing::ScopeStack::new();

            let theme = highlighting::ThemeSet::load_defaults()
                .themes
                .remove("base16-mocha.dark")
                .unwrap();

            Highlighter {
                syntaxes,
                parser,
                stack,
                theme,
                token: settings.token.clone(),
                current_line: 0,
            }
        }

        fn change_line(&mut self, _line: usize) {
            // TODO: Caching
            let syntax = self
                .syntaxes
                .find_syntax_by_token(&self.token)
                .unwrap_or_else(|| self.syntaxes.find_syntax_plain_text());

            self.parser = parsing::ParseState::new(&syntax);
            self.stack = parsing::ScopeStack::new();
            self.current_line = 0;
        }

        fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
            self.current_line += 1;

            let ops = self
                .parser
                .parse_line(line, &self.syntaxes)
                .unwrap_or_default();

            Box::new(
                ScopeRangeIterator {
                    ops,
                    line_length: line.len(),
                    index: 0,
                    last_str_index: 0,
                }
                .filter_map(move |(range, scope)| {
                    let highlighter =
                        highlighting::Highlighter::new(&self.theme);
                    let _ = self.stack.apply(&scope);

                    if range.is_empty() {
                        None
                    } else {
                        Some((
                            range,
                            Highlight(
                                highlighter
                                    .style_mod_for_stack(&self.stack.scopes),
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
