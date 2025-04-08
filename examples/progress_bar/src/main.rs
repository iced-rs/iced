use iced::Element;
use iced::widget::{
    center, center_x, checkbox, column, progress_bar, row, slider,
    vertical_slider,
};

pub fn main() -> iced::Result {
    iced::run(Progress::update, Progress::view)
}

#[derive(Default)]
struct Progress {
    value: f32,
    is_vertical: bool,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    SliderChanged(f32),
    ToggleVertical(bool),
}

impl Progress {
    fn update(&mut self, message: Message) {
        match message {
            Message::SliderChanged(x) => self.value = x,
            Message::ToggleVertical(is_vertical) => {
                self.is_vertical = is_vertical
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let bar = progress_bar(0.0..=100.0, self.value);

        column![
            if self.is_vertical {
                center(
                    row![
                        bar.vertical(),
                        vertical_slider(
                            0.0..=100.0,
                            self.value,
                            Message::SliderChanged
                        )
                        .step(0.01)
                    ]
                    .spacing(20),
                )
            } else {
                center(
                    column![
                        bar,
                        slider(0.0..=100.0, self.value, Message::SliderChanged)
                            .step(0.01)
                    ]
                    .spacing(20),
                )
            },
            center_x(
                checkbox("Vertical", self.is_vertical)
                    .on_toggle(Message::ToggleVertical)
            ),
        ]
        .spacing(20)
        .padding(20)
        .into()
    }
}
