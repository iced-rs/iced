use iced::widget::{button, column, radio, text_input, Toggler};
use iced::{Element, Sandbox, Settings};

pub fn main() -> iced::Result {
    Animations::run(Settings::default())
}

struct Animations {
    _animation_multiplier: i32,
    radio1: Option<usize>,
    radio2: Option<usize>,
    input_text: String,
    toggled: bool,
}

#[derive(Debug, Clone)]
enum Message {
    ButtonPressed,
    RadioPressed(usize),
    TextSubmitted,
    TextChanged(String),
    Toggle(bool),
}

impl Sandbox for Animations {
    type Message = Message;

    fn new() -> Self {
        Self {
            _animation_multiplier: 0,
            radio1: None,
            radio2: None,
            input_text: "".into(),
            toggled: false,
        }
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }

    fn title(&self) -> String {
        String::from("Counter - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::RadioPressed(i) => match i {
                1 => {
                    self.radio1 = Some(1);
                    self.radio2 = None;
                }
                2 => {
                    self.radio2 = Some(2);
                    self.radio1 = None;
                }
                _ => {}
            },
            Message::TextChanged(txt) => {
                self.input_text = txt;
            }
            Message::Toggle(t) => self.toggled = t,
            Message::TextSubmitted | Message::ButtonPressed => {}
        }
    }

    fn view(&self) -> Element<Message> {
        column![
            text_input("Insert some text here...", &self.input_text)
                .on_submit(Message::TextSubmitted)
                .on_input(|txt| Message::TextChanged(txt)),
            button("Press me").on_press(Message::ButtonPressed),
            radio("Click me 1", 1, self.radio1, |i| {
                Message::RadioPressed(i)
            }),
            radio("Click me 2", 2, self.radio2, |i| {
                Message::RadioPressed(i)
            }),
            Toggler::new(Some("Toggle me".into()), self.toggled, |t| {
                Message::Toggle(t)
            })
        ]
        .spacing(10)
        .padding(50)
        .max_width(300)
        .into()
    }
}
