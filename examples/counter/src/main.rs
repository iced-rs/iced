use iced::widget::{
    Column, button, column, container, horizontal_space, row, slider, text,
};
use iced::{Center, Fill};

pub fn main() -> iced::Result {
    iced::run(Counter::update, Counter::view)
}

#[derive(Default)]
struct Counter {
    value: i64,
    width: f32,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Increment,
    Decrement,
    WidthChanged(f32),
}

impl Counter {
    fn update(&mut self, message: Message) {
        match message {
            Message::Increment => {
                self.value += 1;
            }
            Message::Decrement => {
                self.value -= 1;
            }
            Message::WidthChanged(width) => {
                self.width = width;
            }
        }
    }

    fn view(&self) -> Column<Message> {
        column![
            row![
                container(horizontal_space()).width(self.width),
                column![
                    row![
                        button("Increment").on_press(Message::Increment),
                        button("Increment").on_press(Message::Increment),
                        button("Increment").on_press(Message::Increment),
                    ]
                    .spacing(10),
                    text(self.value).size(50),
                    button(container("Decrement").center_x(Fill))
                        .on_press(Message::Decrement)
                        .width(200)
                ]
                .padding(20)
                .align_x(Center)
            ],
            slider(0.0..=1.0, self.width, Message::WidthChanged).step(0.01),
            text!("{:.2}px", self.width)
        ]
        .spacing(10)
        .padding(10)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use iced_test::{Error, simulator};

//     #[test]
//     fn it_counts() -> Result<(), Error> {
//         let mut counter = Counter { value: 0 };
//         let mut ui = simulator(counter.view());

//         let _ = ui.click("Increment")?;
//         let _ = ui.click("Increment")?;
//         let _ = ui.click("Decrement")?;

//         for message in ui.into_messages() {
//             counter.update(message);
//         }

//         assert_eq!(counter.value, 1);

//         let mut ui = simulator(counter.view());
//         assert!(ui.find("1").is_ok(), "Counter should display 1!");

//         Ok(())
//     }
// }
