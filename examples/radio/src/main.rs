use iced::widget::{column, container, Radio};
use iced::{Element, Length, Sandbox, Settings};

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

#[derive(Default)]
struct Example {
    selected_radio: Option<Choice>,
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
                self.selected_radio = Some(choice);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let selected_radio = Some(Choice::A);

	    Radio::new(Choice::A, "This is A", selected_radio, Message::RadioSelected);
        Radio::new(Choice::B, "This is B", selected_radio, Message::RadioSelected);

        let content = column![selected_radio].spacing(22);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Choice {
    #[default]
    A,
    B,
}

impl Choice {
    const ALL: [Choice; 2] = [
        Choice::A,
        Choice::B,
    ];
}

impl std::fmt::Display for Choice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Choice::A => "A",
                Choice::B => "B",
            }
        )
    }
}
