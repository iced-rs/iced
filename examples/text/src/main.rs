use iced::event;
use iced::widget::{center, column, pick_list, right, stack, text};
use iced::window;
use iced::{Element, Event, Subscription, Task};

pub fn main() -> iced::Result {
    iced::application(Text::new, Text::update, Text::view)
        .subscription(Text::subscription)
        .run()
}

struct Text {
    scale_factor: f32,
    font: Font,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    WindowRescaled(f32),
    FontChanged(Font),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Font {
    SansSerif,
    Serif,
    Monospace,
}

impl Text {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                font: Font::SansSerif,
                scale_factor: 1.0,
            },
            window::latest()
                .and_then(window::scale_factor)
                .map(Message::WindowRescaled),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowRescaled(scale_factor) => {
                self.scale_factor = scale_factor;

                Task::none()
            }
            Message::FontChanged(font) => {
                self.font = font;

                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _, _| {
            let Event::Window(window::Event::Rescaled(scale_factor)) = event else {
                return None;
            };

            Some(Message::WindowRescaled(scale_factor))
        })
    }

    fn view(&self) -> Element<'_, Message> {
        let sizes = 5..=32;

        let font_selector = pick_list(
            [Font::SansSerif, Font::Serif, Font::Monospace],
            Some(self.font),
            Message::FontChanged,
        );

        stack![
            center(
                column(sizes.map(|physical_size| {
                    let size = physical_size as f32 / self.scale_factor;

                    text!(
                        "The quick brown fox jumps over the \
                        lazy dog ({physical_size}px)"
                    )
                    .font(match self.font {
                        Font::SansSerif => iced::Font::DEFAULT,
                        Font::Serif => iced::Font {
                            family: iced::font::Family::Serif,
                            ..iced::Font::DEFAULT
                        },
                        Font::Monospace => iced::Font::MONOSPACE,
                    })
                    .size(size)
                    .into()
                }))
                .spacing(10)
            ),
            right(font_selector).padding(10)
        ]
        .into()
    }
}

impl std::fmt::Display for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Font::SansSerif => "Sans Serif",
            Font::Serif => "Serif",
            Font::Monospace => "Monospace",
        })
    }
}
