use iced::widget::{column, container, radio};
use iced::{Element, Length, Sandbox, Settings};

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

#[derive(Default)]
struct Example {
    radio: Option<Choice>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    RadioSelected(Choice),
}

impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        Default::default()
    }

    fn title(&self) -> String {
        String::from("Radio - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::RadioSelected(value) => {
                self.radio = Some(value);
            }
        }
    }

    fn view(&self) -> Element<Message> {
	let a_checkbox = radio(
            "A",
            Choice::A,
            self.radio,
            Message::RadioSelected,
        );

        let b_checkbox = radio(
	    "B",
            Choice::B,
            self.radio,
            Message::RadioSelected,
        );

        let c_checkbox = radio(
            "C",
            Choice::C,
            self.radio,
            Message::RadioSelected,
        );

        let all_checkbox = radio("All of the above", Choice::All, self.radio, Message::RadioSelected);

        let content = column![
            a_checkbox,
            b_checkbox,
            c_checkbox,
            all_checkbox,
        ]
	.spacing(20)
        .padding(20)
        .max_width(600);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Choice {
    A,
    B,
    C,
    All,
}
