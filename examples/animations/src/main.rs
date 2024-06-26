use iced::{
    widget::{
        button, checkbox, column, container, row, scrollable, text, text_input,
        Toggler,
    },
    Element, Length, Task, Theme,
};
use once_cell::sync::Lazy;
use std::fmt::Write;

pub fn main() -> iced::Result {
    iced::application("Animated widgets", Animations::update, Animations::view)
        .theme(Animations::theme)
        .run()
}

#[derive(Default)]
struct Animations {
    input_text: String,
    toggled: bool,
    checked: bool,
    animations_disabled: bool,
}

#[derive(Debug, Clone)]
enum Message {
    ButtonPressed,
    TextSubmitted,
    TextChanged(String),
    Toggle(bool),
    Check(bool),
    DisableAnimations(bool),
    GoToStart,
    GoToEnd,
}

static SCROLLABLE_TEXT: Lazy<String> = Lazy::new(|| {
    (0..50).fold(String::new(), |mut output, i| {
        let _ = writeln!(
            output,
            "This is a long text string to test the scrollbar: {i}\n"
        );
        output
    })
});
static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);

impl Animations {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TextChanged(txt) => {
                self.input_text = txt;
                Task::none()
            }
            Message::Toggle(t) => {
                self.toggled = t;
                Task::none()
            }
            Message::TextSubmitted | Message::ButtonPressed => Task::none(),
            Message::Check(c) => {
                self.checked = c;
                Task::none()
            }
            Message::DisableAnimations(b) => {
                self.animations_disabled = b;
                Task::none()
            }
            Message::GoToStart => scrollable::snap_to(
                SCROLLABLE_ID.clone(),
                scrollable::RelativeOffset::START,
            ),
            Message::GoToEnd => scrollable::snap_to(
                SCROLLABLE_ID.clone(),
                scrollable::RelativeOffset::END,
            ),
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
            checkbox("Check me", self.checked)
                .on_toggle(|t| { Message::Check(t) }),
            Toggler::new(Some("Toggle me".into()), self.toggled, |t| {
                Message::Toggle(t)
            }),
            scrollable(
                container(column![
                    button("Go to end").on_press(Message::GoToEnd),
                    text(SCROLLABLE_TEXT.as_str()),
                    button("Go to start").on_press(Message::GoToStart)
                ])
                .width(Length::Fill)
                .padding(5)
            )
            .id(SCROLLABLE_ID.clone())
        ]
        .spacing(10)
        .padding(50)
        .max_width(800)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Light
    }
}
