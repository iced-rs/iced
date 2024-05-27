use iced::{
    widget::{
        button, checkbox, column, container, radio, row, text_input, Toggler,
    },
    Element,
};

pub fn main() -> iced::Result {
    iced::run("Animated widgets", Animations::update, Animations::view)
}

#[derive(Default)]
struct Animations {
    _animation_multiplier: i32,
    radio1: Option<usize>,
    radio2: Option<usize>,
    input_text: String,
    toggled: bool,
    checked: bool,
    animations_disabled: bool,
}

#[derive(Debug, Clone)]
enum Message {
    ButtonPressed,
    RadioPressed(usize),
    TextSubmitted,
    TextChanged(String),
    Toggle(bool),
    Check(bool),
    DisableAnimations(bool),
}

impl Animations {
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
            Message::Check(c) => self.checked = c,
            Message::DisableAnimations(b) => self.animations_disabled = b,
        }
    }

    fn view(&self) -> Element<Message> {
        column![
            text_input("Insert some text here...", &self.input_text)
                .on_submit(Message::TextSubmitted)
                .on_input(Message::TextChanged),
            row![
                button("Primary")
                    .on_press(Message::ButtonPressed)
                    .style(button::primary)
                    .set_animations_enabled(!self.animations_disabled),
                button("Secondary")
                    .on_press(Message::ButtonPressed)
                    .style(button::secondary)
                    .set_animations_enabled(!self.animations_disabled),
                button("Success")
                    .on_press(Message::ButtonPressed)
                    .style(button::success)
                    .set_animations_enabled(!self.animations_disabled),
                button("Danger")
                    .on_press(Message::ButtonPressed)
                    .style(button::danger)
                    .set_animations_enabled(!self.animations_disabled),
                container(Toggler::new(
                    Some("Disable buttons animations".into()),
                    self.animations_disabled,
                    Message::DisableAnimations,
                ))
                .padding(5)
            ],
            button("Text")
                .on_press(Message::ButtonPressed)
                .style(button::text),
            radio("Click me 1", 1, self.radio1, |i| {
                Message::RadioPressed(i)
            }),
            radio("Click me 2", 2, self.radio2, |i| {
                Message::RadioPressed(i)
            }),
            checkbox("Check me", self.checked)
                .on_toggle(|t| { Message::Check(t) }),
            Toggler::new(Some("Toggle me".into()), self.toggled, |t| {
                Message::Toggle(t)
            }),
        ]
        .spacing(10)
        .padding(50)
        .max_width(800)
        .into()
    }
}
