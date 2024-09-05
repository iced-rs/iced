use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, container, row, text, vertical_rule, vertical_space,
        Container,
    },
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
enum Tool {
    Pencil,
    Eraser,
    Text,
    #[default]
    Brush,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum Action {
    #[default]
    Draw,
    Select,
    Tool(Tool),
}

#[derive(Debug, Clone)]
enum Message {
    Selector,
    Tool(Tool),
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

        let tools = {
            let tool_btn = |code: char, message: Message| {
                let icon = text(code).font(ICON_FONT);

                button(icon).on_press(message).style(styles::toolbar_btn)
            };

            let rw1 = row!(
                tool_btn('\u{E800}', Message::Tool(Tool::Pencil)),
                tool_btn('\u{F12D}', Message::Tool(Tool::Eraser))
            );
            let rw2 = row!(
                tool_btn('\u{E801}', Message::Tool(Tool::Text)),
                tool_btn('\u{F1FC}', Message::Tool(Tool::Brush))
            );

            let description = text("Tools");

            let tools = column!(rw1, rw2);

            column!(tools, vertical_space(), description)
                .align_x(Horizontal::Center)
                .width(75)
                .height(Length::Fill)
        };

        container(
            row!(selector, vertical_rule(2), tools)
                .width(Length::Fill)
                .height(Length::Fixed(100.0))
                .spacing(7.0)
                .padding([5, 8])
                .align_y(Vertical::Center),
        )
        .style(styles::toolbar)
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Selector => self.action = Action::Select,
            Message::Tool(tool) => self.action = Action::Tool(tool),
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
