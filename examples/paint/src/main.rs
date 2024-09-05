use iced::{
    alignment::Vertical,
    widget::{container, row, text, Container},
    Element, Length, Task, Theme,
};

fn main() -> iced::Result {
    iced::application("Iced Paint", Paint::update, Paint::view)
        .theme(|_| Theme::TokyoNight)
        .antialiasing(true)
        .run_with(init)
}

fn init() -> (Paint, Task<Message>) {
    (Paint::new(), Task::none())
}

#[derive(Debug, Clone)]
enum Message {}

#[derive(Debug, Default)]
struct Paint {}

impl Paint {
    fn new() -> Self {
        Paint {}
    }

    fn toolbar(&self) -> Container<'_, Message> {
        let text = text("Paint here");

        container(
            row!(text)
                .width(Length::Fill)
                .height(Length::Fixed(125.0))
                .padding([5, 8])
                .align_y(Vertical::Center),
        )
        .style(styles::toolbar)
    }

    fn update(&mut self, _message: Message) {}

    fn view(&self) -> Element<Message> {
        container(self.toolbar()).into()
    }
}

mod styles {
    use iced::{widget, Background, Theme};

    pub fn toolbar(theme: &Theme) -> widget::container::Style {
        let background = theme.extended_palette().background.weak;

        widget::container::Style {
            background: Some(Background::Color(background.color)),
            text_color: Some(background.text),
            ..Default::default()
        }
    }
}
