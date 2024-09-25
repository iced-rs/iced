use iced::widget::{
    center, column, pick_list, qr_code, row, slider, text, text_input, toggler,
};
use iced::{Center, Element, Theme};

const QR_CODE_EXACT_SIZE_MIN_PX: u32 = 200;
const QR_CODE_EXACT_SIZE_MAX_PX: u32 = 400;
const QR_CODE_EXACT_SIZE_SLIDER_STEPS: u8 = 100;

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
    display_with_fixed_size: bool,
    fixed_size_slider_value: u8,
    theme: Theme,
}

#[derive(Debug, Clone)]
enum Message {
    DataChanged(String),
    SetDisplayWithFixedSize(bool),
    FixedSizeSliderChanged(u8),
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
            Message::SetDisplayWithFixedSize(exact_size) => {
                self.display_with_fixed_size = exact_size;
            }
            Message::FixedSizeSliderChanged(value) => {
                self.fixed_size_slider_value = value;
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
            .push(
                toggler(self.display_with_fixed_size)
                    .on_toggle(Message::SetDisplayWithFixedSize)
                    .label("Fixed Size"),
            )
            .push_maybe(self.display_with_fixed_size.then(|| {
                slider(
                    1..=QR_CODE_EXACT_SIZE_SLIDER_STEPS,
                    self.fixed_size_slider_value,
                    Message::FixedSizeSliderChanged,
                )
            }))
            .push_maybe(self.qr_code.as_ref().map(|data| {
                if self.display_with_fixed_size {
                    // Convert the slider value to a size in pixels.
                    let qr_code_size_px = (self.fixed_size_slider_value as f32
                        / QR_CODE_EXACT_SIZE_SLIDER_STEPS as f32)
                        * (QR_CODE_EXACT_SIZE_MAX_PX
                            - QR_CODE_EXACT_SIZE_MIN_PX)
                            as f32
                        + QR_CODE_EXACT_SIZE_MIN_PX as f32;

                    qr_code(data).total_size(qr_code_size_px)
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
