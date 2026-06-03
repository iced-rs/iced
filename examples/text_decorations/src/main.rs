//! Demonstrates rich-text decorations (underline, strikethrough, and highlight
//! backgrounds) rendered live in a `text_editor` via a custom [`Highlighter`].
//!
//! The decorations are carried as cosmic-text glyph attributes and drawn by the
//! iced renderers. Underline + highlight + bold render on both the wgpu and
//! tiny-skia backends; strikethrough currently renders on the tiny-skia
//! (software) backend — run with `ICED_BACKEND=tiny-skia cargo run` to see it.
use std::ops::Range;

use iced::advanced::text::highlighter::{self, Highlighter};
use iced::widget::{column, container, text, text_editor};
use iced::{Color, Element, Fill, Font, font};

const SAMPLE: &str = "\
Rich-text decorations, rendered live by cosmic-text + iced.

The word underline is underlined.
The word strikethrough has a line through it.
The word highlight sits on a colored background.
The word bold is shown bold and colored.

Edit freely — decorations update as you type.";

pub fn main() -> iced::Result {
    iced::application(Demo::new, Demo::update, Demo::view)
        .default_font(Font::MONOSPACE)
        .run()
}

struct Demo {
    content: text_editor::Content,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
}

impl Demo {
    fn new() -> Self {
        Self {
            content: text_editor::Content::with_text(SAMPLE),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Edit(action) => self.content.perform(action),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let editor = text_editor(&self.content)
            .placeholder("Type something...")
            .on_action(Message::Edit)
            .height(Fill)
            .highlight_with::<DemoHighlighter>((), to_format);

        container(
            column![
                text("Rich-text decorations (underline · strikethrough · highlight · bold)")
                    .size(18),
                editor,
            ]
            .spacing(12),
        )
        .padding(20)
        .into()
    }
}

/// The decoration applied to a matched word.
#[derive(Clone, Copy)]
enum Decoration {
    Underline,
    Strikethrough,
    Highlight,
    Bold,
}

/// A tiny highlighter that decorates a few demo keywords wherever they appear.
struct DemoHighlighter {
    current_line: usize,
}

impl Highlighter for DemoHighlighter {
    type Settings = ();
    type Highlight = Decoration;
    type Iterator<'a> = std::vec::IntoIter<(Range<usize>, Decoration)>;

    fn new(_settings: &Self::Settings) -> Self {
        Self { current_line: 0 }
    }

    fn update(&mut self, _new_settings: &Self::Settings) {}

    fn change_line(&mut self, line: usize) {
        self.current_line = line;
    }

    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        self.current_line += 1;

        const KEYWORDS: [(&str, Decoration); 4] = [
            ("underline", Decoration::Underline),
            ("strikethrough", Decoration::Strikethrough),
            ("highlight", Decoration::Highlight),
            ("bold", Decoration::Bold),
        ];

        let mut spans: Vec<(Range<usize>, Decoration)> = Vec::new();
        for (keyword, decoration) in KEYWORDS {
            let mut from = 0;
            while let Some(offset) = line[from..].find(keyword) {
                let start = from + offset;
                spans.push((start..start + keyword.len(), decoration));
                from = start + keyword.len();
            }
        }
        spans.sort_by_key(|(range, _)| range.start);
        spans.into_iter()
    }

    fn current_line(&self) -> usize {
        self.current_line
    }
}

fn to_format(decoration: &Decoration, _theme: &iced::Theme) -> highlighter::Format<Font> {
    match decoration {
        Decoration::Underline => highlighter::Format {
            underline: true,
            color: Some(Color::from_rgb(0.29, 0.56, 0.85)),
            ..highlighter::Format::default()
        },
        Decoration::Strikethrough => highlighter::Format {
            strikethrough: true,
            color: Some(Color::from_rgb(0.85, 0.34, 0.34)),
            ..highlighter::Format::default()
        },
        Decoration::Highlight => highlighter::Format {
            background: Some(Color::from_rgb(1.0, 0.95, 0.69)),
            color: Some(Color::BLACK),
            ..highlighter::Format::default()
        },
        Decoration::Bold => highlighter::Format {
            font: Some(Font {
                weight: font::Weight::Bold,
                ..Font::MONOSPACE
            }),
            color: Some(Color::from_rgb(0.55, 0.36, 0.96)),
            ..highlighter::Format::default()
        },
    }
}
