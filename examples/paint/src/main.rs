use iced::{
    alignment::{Horizontal, Vertical},
    widget::{button, column, container, row, text, vertical_space, Container},
    Element, Font, Length, Theme,
};

const ICON_FONT: Font = Font::with_name("paint-icons");

fn main() -> iced::Result {
    iced::application("Iced Paint", Paint::update, Paint::view)
        .theme(|_| Theme::TokyoNight)
        .antialiasing(true)
        .font(include_bytes!("../fonts/paint-icons.ttf").as_slice())
        .run()
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum Action {
    #[default]
    Draw,
    Select,
}

#[derive(Debug, Clone)]
enum Message {
    Selector,
}

#[derive(Debug, Default)]
struct Paint {
    action: Action,
}

impl Paint {
    fn toolbar(&self) -> Container<'_, Message> {
        let selector = {
            let icon = text('\u{E847}').size(40.0).font(ICON_FONT);

            let btn = button(icon)
                .on_press(Message::Selector)
                .padding([2, 6])
                .style(styles::toolbar_btn);

            let description = text("Selection");

            column!(btn, vertical_space(), description)
                .align_x(Horizontal::Center)
                .width(75)
                .height(Length::Fill)
        };

        container(
            row!(selector)
                .width(Length::Fill)
                .height(Length::Fixed(100.0))
                .spacing(5.0)
                .padding([5, 8])
                .align_y(Vertical::Center),
        )
        .style(styles::toolbar)
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Selector => self.action = Action::Select,
        }
    }

    fn view(&self) -> Element<Message> {
        container(self.toolbar()).into()
    }
}

mod styles {
    use iced::{widget, Background, Border, Theme};

    pub fn toolbar(theme: &Theme) -> widget::container::Style {
        let background = theme.extended_palette().background.weak;

        widget::container::Style {
            background: Some(Background::Color(background.color)),
            text_color: Some(background.text),
            ..Default::default()
        }
    }

    pub fn toolbar_btn(
        theme: &Theme,
        status: widget::button::Status,
    ) -> widget::button::Style {
        match status {
            widget::button::Status::Hovered => {
                let background = theme.extended_palette().background.strong;

                widget::button::Style {
                    background: Some(Background::Color(background.color)),
                    border: Border {
                        radius: 5.0.into(),
                        ..Default::default()
                    },
                    text_color: background.text,
                    ..Default::default()
                }
            }
            _ => {
                let background = theme.extended_palette().background.weak;

                widget::button::Style {
                    background: Some(Background::Color(background.color)),
                    border: Border {
                        radius: 5.0.into(),
                        ..Default::default()
                    },
                    text_color: background.text,
                    ..Default::default()
                }
            }
        }
    }
}
