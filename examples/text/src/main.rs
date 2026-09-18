use iced::event;
use iced::font;
use iced::widget::{center, column, pick_list, right, stack, text};
use iced::window;
use iced::{Element, Event, Font, Subscription, Task};

pub fn main() -> iced::Result {
    iced::application(Text::new, Text::update, Text::view)
        .subscription(Text::subscription)
        .run()
}

struct Text {
    scale_factor: f32,
    font: Font,
    families: Vec<font::Family>,
}

#[derive(Debug, Clone)]
enum Message {
    WindowRescaled(f32),
    FontChanged(font::Family),
    FontsListed(Vec<font::Family>),
}

impl Text {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                font: Font::DEFAULT,
                scale_factor: 1.0,
                families: font::Family::VARIANTS.to_vec(),
            },
            Task::batch([
                window::latest()
                    .and_then(window::scale_factor)
                    .map(Message::WindowRescaled),
                font::list()
                    .map(Result::ok)
                    .and_then(Task::done)
                    .map(Message::FontsListed),
            ]),
        )
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::WindowRescaled(scale_factor) => {
                self.scale_factor = scale_factor;
            }
            Message::FontChanged(family) => {
                self.font = Font::with_family(family);
            }
            Message::FontsListed(families) => {
                self.families = families
                    .into_iter()
                    .chain(font::Family::VARIANTS.iter().copied())
                    .collect();
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
            Some(self.font.family),
            self.families.as_slice(),
            font::Family::to_string,
        )
        .on_select(Message::FontChanged);

        stack![
            center(
                column(sizes.map(|physical_size| {
                    let size = physical_size as f32 / self.scale_factor;

                    text!(
                        "The quick brown fox jumps over the \
                        lazy dog ({physical_size}px)"
                    )
                    .font(self.font)
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
