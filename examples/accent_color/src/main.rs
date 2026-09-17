use iced::border;
use iced::system;
use iced::theme::{self, palette};
use iced::widget::{button, center, column, container, progress_bar, row, text, toggler};
use iced::{Center, Color, Element, Fill, Fit, Subscription, Task, Theme};

pub fn main() -> iced::Result {
    iced::application(Example::new, Example::update, Example::view)
        .subscription(Example::subscription)
        .theme(Example::theme)
        .run()
}

#[derive(Default)]
struct Example {
    accent_color: Option<Color>,
    mode: theme::Mode,
    is_toggled: bool,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    AccentColorChanged(Option<Color>),
    ThemeChanged(theme::Mode),
    Toggled(bool),
}

impl Example {
    fn new() -> (Self, Task<Message>) {
        (
            Self::default(),
            Task::batch([
                system::accent_color().map(Message::AccentColorChanged),
                system::theme().map(Message::ThemeChanged),
            ]),
        )
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::AccentColorChanged(accent_color) => {
                self.accent_color = accent_color;
            }
            Message::ThemeChanged(mode) => {
                self.mode = mode;
            }
            Message::Toggled(is_toggled) => {
                self.is_toggled = is_toggled;
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            system::accent_color_changes().map(Message::AccentColorChanged),
            system::theme_changes().map(Message::ThemeChanged),
        ])
    }

    fn view(&self) -> Element<'_, Message> {
        let swatch = container(
            text(match self.accent_color {
                Some(accent_color) => accent_color.to_string(),
                None => String::from("No accent color"),
            })
            .size(30),
        )
        .style(|theme: &Theme| {
            let primary = theme.palette().primary.base;

            container::background(primary.color)
                .color(primary.text)
                .border(border::rounded(8))
        })
        .center_x(Fill)
        .center_y(150);

        let controls = row![
            button("Press me").on_press(Message::Toggled(!self.is_toggled)),
            toggler(self.is_toggled)
                .label("Toggle me")
                .on_toggle(Message::Toggled),
            progress_bar(0.0..=1.0, if self.is_toggled { 1.0 } else { 0.3 }),
        ]
        .spacing(20)
        .align_y(Center);

        center(
            column![swatch, text!("System theme: {:?}", self.mode), controls]
                .spacing(20)
                .padding(20)
                .width(Fit.max(600)),
        )
        .into()
    }

    fn theme(&self) -> Theme {
        let base = <Theme as theme::Base>::default(self.mode);

        let Some(accent_color) = self.accent_color else {
            return base;
        };

        Theme::custom(
            format!("{base} ({accent_color})"),
            palette::Seed {
                primary: accent_color,
                ..base.seed()
            },
        )
    }
}
