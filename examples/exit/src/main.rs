use iced::widget::{button, center, column};
use iced::window;
use iced::{Center, Element, Task};

pub fn main() -> iced::Result {
    iced::application("Exit - Iced", Exit::update, Exit::view).run()
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
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Confirm => window::get_latest().and_then(window::close),
            Message::Exit => {
                self.show_confirm = true;

                Task::none()
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
        .align_x(Center);

        center(content).padding(20).into()
    }
}
