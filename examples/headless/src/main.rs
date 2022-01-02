use iced::{
    button, window::take_screenshot, Alignment, Application, Button, Column,
    Command, Element, Screenshot, Settings, Text,
};

use std::time::Duration;

pub fn main() -> iced::Result {
    let msg_trace = vec![
        (Message::IncrementPressed, Duration::new(1, 0)),
        (Message::IncrementPressed, Duration::new(2, 0)),
        (Message::IncrementPressed, Duration::new(2, 0)),
        (Message::TakeScreenshot, Duration::new(3, 0)),
    ];
    Counter::run_with_message_trace(
        Settings {
            headless: true,
            window: iced::window::Settings {
                size: (600, 600),
                ..iced::window::Settings::default()
            },

            ..Settings::default()
        },
        msg_trace,
    )
}

#[derive(Default)]
struct Counter {
    value: i32,
    increment_button: button::State,
    decrement_button: button::State,
    screenshot: button::State,
}

#[derive(Debug, Clone)]
enum Message {
    IncrementPressed,
    DecrementPressed,
    TakeScreenshot,
    ScreenshotReceiver(Option<Screenshot>),
}

impl Application for Counter {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Counter - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::IncrementPressed => {
                self.value += 1;
            }
            Message::DecrementPressed => {
                self.value -= 1;
            }
            Message::TakeScreenshot => {
                return take_screenshot(Box::new(Message::ScreenshotReceiver));
            }
            Message::ScreenshotReceiver(ss) => {
                ss.map(|ss| {
                    ss.save_image_to_png(format!("counter_{}.png", self.value))
                });
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        Column::new()
            .padding(20)
            .align_items(Alignment::Center)
            .push(
                Button::new(&mut self.increment_button, Text::new("Increment"))
                    .on_press(Message::IncrementPressed),
            )
            .push(Text::new(self.value.to_string()).size(50))
            .push(
                Button::new(&mut self.decrement_button, Text::new("Decrement"))
                    .on_press(Message::DecrementPressed),
            )
            .push(
                Button::new(&mut self.screenshot, Text::new("Screenshot me!"))
                    .on_press(Message::TakeScreenshot),
            )
            .into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    /// We start up the [`Counter`] applcation and take a screenshot. Then we compare with a golden
    /// value
    fn base_test() {
        let msg_trace = vec![(Message::TakeScreenshot, Duration::new(1, 0))];
        Counter::run_with_message_trace(
            Settings {
                headless: true,
                window: iced::window::Settings {
                    size: (600, 600),
                    ..iced::window::Settings::default()
                },

                ..Settings::default()
            },
            msg_trace,
        )
        .expect("Running failed");


        // get the golden png into a screenshot object
        let path_to_base_golden =
            format!("{}/golden/base_counter.png", env!("CARGO_MANIFEST_DIR"));

        let golden = Screenshot::from_png(path_to_base_golden)
            .expect("Png conversion failed for golden");


        // get the png that is created by the application, put into a screenshot object
        let screenshot_png =
            format!("{}/counter_0.png", env!("CARGO_MANIFEST_DIR"));

        let png_test =
            Screenshot::from_png(screenshot_png.clone()).expect("png test");

        // try to remove the screenshot, dont let this cause the test to fail though
        let _ = std::fs::remove_file(screenshot_png);

        //compare the golden object and the png test object
        assert_eq!(golden, png_test);
    }
}
