use iced::widget::{center, column, pick_list, qr_code, row, text, text_input};
use iced::{Center, Element, Theme};

pub fn main() -> iced::Result {
    iced::application(
        "QR Code Generator - Iced",
        QRGenerator::update,
        QRGenerator::view,
    )
    .theme(QRGenerator::theme)
    .run()
}

#[derive(Default)]
struct QRGenerator {
    data: String,
    qr_code: Option<qr_code::Data>,
    theme: Theme,
}

#[derive(Debug, Clone)]
enum Message {
    DataChanged(String),
    ThemeChanged(Theme),
}

impl QRGenerator {
    fn update(&mut self, message: Message) {
        match message {
            Message::DataChanged(mut data) => {
                data.truncate(100);

                self.qr_code = if data.is_empty() {
                    None
                } else {
                    qr_code::Data::new(&data).ok()
                };

                self.data = data;
            }
            Message::ThemeChanged(theme) => {
                self.theme = theme;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let title = text("QR Code Generator").size(70);

        let input =
            text_input("Type the data of your QR code here...", &self.data)
                .on_input(Message::DataChanged)
                .size(30)
                .padding(15);

        let choose_theme = row![
            text("Theme:"),
            pick_list(Theme::ALL, Some(&self.theme), Message::ThemeChanged,)
        ]
        .spacing(10)
        .align_y(Center);

        let content = column![title, input, choose_theme]
            .push_maybe(
                self.qr_code
                    .as_ref()
                    .map(|data| qr_code(data).cell_size(10)),
            )
            .width(700)
            .spacing(20)
            .align_x(Center);

        center(content).padding(20).into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
