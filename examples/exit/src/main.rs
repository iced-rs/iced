use iced::widget::{button, column, container};
use iced::window;
use iced::{Alignment, Command, Element, Length};

pub fn main() -> iced::Result {
    iced::application(Exit::new, Exit::update, Exit::view)
        .title("Exit - Iced")
        .run()
}

#[derive(Default)]
struct Exit {
    show_confirm: bool,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Confirm,
    Exit,
}

impl Exit {
    fn new() -> (Self, Command<Message>) {
        (Self::default(), Command::none())
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Confirm => window::close(window::Id::MAIN),
            Message::Exit => {
                self.show_confirm = true;

                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let content = if self.show_confirm {
            column![
                "Are you sure you want to exit?",
                button("Yes, exit now")
                    .padding([10, 20])
                    .on_press(Message::Confirm),
            ]
        } else {
            column![
                "Click the button to exit",
                button("Exit").padding([10, 20]).on_press(Message::Exit),
            ]
        }
        .spacing(10)
        .align_items(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }
}
