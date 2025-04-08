use iced::widget::{
    center, column, pick_list, qr_code, row, slider, text, text_input, toggler,
};
use iced::{Center, Element, Theme};

use std::ops::RangeInclusive;

pub fn main() -> iced::Result {
    iced::application(
        QRGenerator::default,
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
    total_size: Option<f32>,
    theme: Theme,
}

#[derive(Debug, Clone)]
enum Message {
    DataChanged(String),
    ToggleTotalSize(bool),
    TotalSizeChanged(f32),
    ThemeChanged(Theme),
}

impl QRGenerator {
    const SIZE_RANGE: RangeInclusive<f32> = 200.0..=400.0;

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
            Message::ToggleTotalSize(enabled) => {
                self.total_size = enabled.then_some(
                    Self::SIZE_RANGE.start()
                        + (Self::SIZE_RANGE.end() - Self::SIZE_RANGE.start())
                            / 2.0,
                );
            }
            Message::TotalSizeChanged(total_size) => {
                self.total_size = Some(total_size);
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

        let toggle_total_size = toggler(self.total_size.is_some())
            .on_toggle(Message::ToggleTotalSize)
            .label("Limit Total Size");

        let choose_theme = row![
            text("Theme:"),
            pick_list(Theme::ALL, Some(&self.theme), Message::ThemeChanged,)
        ]
        .spacing(10)
        .align_y(Center);

        let content = column![
            title,
            input,
            row![toggle_total_size, choose_theme]
                .spacing(20)
                .align_y(Center)
        ]
        .push_maybe(self.total_size.map(|total_size| {
            slider(Self::SIZE_RANGE, total_size, Message::TotalSizeChanged)
        }))
        .push_maybe(self.qr_code.as_ref().map(|data| {
            if let Some(total_size) = self.total_size {
                qr_code(data).total_size(total_size)
            } else {
                qr_code(data).cell_size(10.0)
            }
        }))
        .width(700)
        .spacing(20)
        .align_x(Center);

        center(content).padding(20).into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
