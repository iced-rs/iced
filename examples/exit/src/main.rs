use iced::{
    button, Alignment, Button, Column, Container, Element, Length, Sandbox,
    Settings, Text,
};

pub fn main() -> iced::Result {
    Exit::run(Settings::default())
}

#[derive(Default)]
struct Exit {
    show_confirm: bool,
    exit: bool,
    confirm_button: button::State,
    exit_button: button::State,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Confirm,
    Exit,
}

impl Sandbox for Exit {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Exit - Iced")
    }

    fn should_exit(&self) -> bool {
        self.exit
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Confirm => {
                self.exit = true;
            }
            Message::Exit => {
                self.show_confirm = true;
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let content = if self.show_confirm {
            Column::new()
                .spacing(10)
                .align_items(Alignment::Center)
                .push(Text::new("Are you sure you want to exit?"))
                .push(
                    Button::new(
                        &mut self.confirm_button,
                        Text::new("Yes, exit now"),
                    )
                    .padding([10, 20])
                    .on_press(Message::Confirm),
                )
        } else {
            Column::new()
                .spacing(10)
                .align_items(Alignment::Center)
                .push(Text::new("Click the button to exit"))
                .push(
                    Button::new(&mut self.exit_button, Text::new("Exit"))
                        .padding([10, 20])
                        .on_press(Message::Exit),
                )
        };

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }
}
