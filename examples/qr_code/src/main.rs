use iced::qr_code::{self, QRCode};
use iced::text_input::{self, TextInput};
use iced::{
    Alignment, Column, Container, Element, Length, Sandbox, Settings, Text,
};

pub fn main() -> iced::Result {
    QRGenerator::run(Settings::default())
}

#[derive(Default)]
struct QRGenerator {
    data: String,
    input: text_input::State,
    qr_code: Option<qr_code::State>,
}

#[derive(Debug, Clone)]
enum Message {
    DataChanged(String),
}

impl Sandbox for QRGenerator {
    type Message = Message;

    fn new() -> Self {
        QRGenerator {
            qr_code: qr_code::State::new("").ok(),
            ..Self::default()
        }
    }

    fn title(&self) -> String {
        String::from("QR Code Generator - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::DataChanged(mut data) => {
                data.truncate(100);

                self.qr_code = qr_code::State::new(&data).ok();
                self.data = data;
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let title = Text::new("QR Code Generator")
            .size(70)
            .color([0.5, 0.5, 0.5]);

        let input = TextInput::new(
            &mut self.input,
            "Type the data of your QR code here...",
            &self.data,
            Message::DataChanged,
        )
        .size(30)
        .padding(15);

        let mut content = Column::new()
            .width(Length::Units(700))
            .spacing(20)
            .align_items(Alignment::Center)
            .push(title)
            .push(input);

        if let Some(qr_code) = self.qr_code.as_mut() {
            content = content.push(QRCode::new(qr_code).cell_size(10));
        }

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }
}
