use iced::{
    widget::{column, Toggler},
    Element, Task, Theme,
};

pub fn main() -> iced::Result {
    iced::application("Animated widgets", Animations::update, Animations::view)
        .theme(Animations::theme)
        .run()
}

#[derive(Default)]
struct Animations {
    toggled: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Toggle(bool),
}

impl Animations {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Toggle(t) => {
                self.toggled = t;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let main_text = iced::widget::text(
            "You can find all widgets with default animations here.",
        );
        let toggle = Toggler::new(self.toggled)
            .label("Toggle me!")
            .on_toggle(Message::Toggle);
        column![main_text, toggle]
            .spacing(10)
            .padding(50)
            .max_width(800)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Light
    }
}
