use iced::widget::{button, column, row, text, text_input};
use iced::{Center, Element, Fill, Task};

pub fn main() -> iced::Result {
    iced::run(VirtualKeyboard::update, VirtualKeyboard::view)
}

#[derive(Default)]
struct VirtualKeyboard {
    input_value: String,
    submitted: String,
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
    Submit,
    Clear,
}

impl VirtualKeyboard {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InputChanged(value) => {
                self.input_value = value;
            }
            Message::Submit => {
                if !self.input_value.is_empty() {
                    self.submitted = self.input_value.clone();
                    self.input_value.clear();
                }
            }
            Message::Clear => {
                self.input_value.clear();
                self.submitted.clear();
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let input = text_input("Type here…", &self.input_value)
            .on_input(Message::InputChanged)
            .on_submit(Message::Submit)
            .padding(12)
            .size(20);

        let submitted_label = if self.submitted.is_empty() {
            text("(nothing submitted yet)")
        } else {
            text(format!("Submitted: {}", self.submitted))
        }
        .size(18);

        let controls = row![
            button("Submit").on_press(Message::Submit),
            button("Clear").on_press(Message::Clear),
        ]
        .spacing(8);

        column![
            text("Virtual Keyboard Test").size(28),
            text("Tap the text box below — the virtual keyboard should appear on mobile.")
                .size(14),
            input,
            controls,
            submitted_label,
        ]
        .spacing(16)
        .padding(24)
        .align_x(Center)
        .width(Fill)
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced_test::{Error, simulator};

    #[test]
    fn it_submits_input() -> Result<(), Error> {
        let mut app = VirtualKeyboard::default();

        // Simulate typing into the text input.
        let _ = app.update(Message::InputChanged("hello".to_owned()));
        assert_eq!(app.input_value, "hello");

        // Submit should move the value to `submitted` and clear the input.
        let _ = app.update(Message::Submit);
        assert_eq!(app.submitted, "hello");
        assert!(app.input_value.is_empty());

        Ok(())
    }

    #[test]
    fn clear_resets_state() -> Result<(), Error> {
        let mut app = VirtualKeyboard {
            input_value: "draft".to_owned(),
            submitted: "old".to_owned(),
        };

        let _ = app.update(Message::Clear);
        assert!(app.input_value.is_empty());
        assert!(app.submitted.is_empty());

        Ok(())
    }

    #[test]
    fn view_renders() -> Result<(), Error> {
        let app = VirtualKeyboard::default();
        let mut ui = simulator(app.view());

        // The text input and both buttons should be present.
        assert!(ui.find("Submit").is_ok());
        assert!(ui.find("Clear").is_ok());

        Ok(())
    }
}
