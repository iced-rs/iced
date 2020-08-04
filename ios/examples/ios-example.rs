//#[macro_use] extern crate log;
use iced::{
    //button, scrollable, slider, text_input, Button, Checkbox, Color,
    Column,
    Sandbox,
    Settings,
    Element,
    Text,
    TextInput,
    Button,
    text_input,
    Color,
};
pub fn main() {
    color_backtrace::install();
    std::env::set_var("RUST_LOG", "DEBUG");
    std::env::set_var("RUST_BACKTRACE", "full");
    pretty_env_logger::init();
    //env_logger::init();

    Simple::run(Settings::default())
}

#[derive(Debug, Default)]
pub struct Simple {
    toggle: bool,
    text: String,
    button_press_count: usize,
    button_state: iced::button::State,
    text_state: text_input::State,
    text_state2: text_input::State,
}
#[derive(Debug, Clone)]
pub enum Message {
    Toggled(bool),
    TextUpdated(String),
    TextSubmit,
    ButtonPress,
}

impl Sandbox for Simple {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Events - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::TextUpdated(val) => {
                self.text = val;
            },
            Message::ButtonPress => {
                self.button_press_count += 1;
            },
            _ => {
            },
        }
    }

    fn view(&mut self) -> Element<Message> {
        if self.text.starts_with("B") {
            TextInput::new(
                &mut self.text_state2,
                "",
                "",
                |s| {
                    Message::TextUpdated(s)
                }
            ).into()
        } else  {
            let mut column = Column::new()
                .push(
                    Text::new(format!("text box: \"{}\", button pressed {:?}", &self.text, self.button_press_count)).color(Color::BLACK)
                )
                .push(
                    Button::new(&mut self.button_state,
                        Text::new(format!("FOO: {}", &self.text)).color(Color::BLACK)
                    ).on_press(Message::ButtonPress)
                )
                .push(
                    TextInput::new(
                        &mut self.text_state,
                        "",
                        "",
                        |s| {
                            Message::TextUpdated(s)
                        }
                    )
                )
                .push(
                    Text::new(format!("BAR: {}", &self.text)).color(Color::BLACK)
                )
                ;
            /*
            if self.text.len() % 2 == 0 {
                column = column.push(
                    Text::new(format!("TEXT IS LENGTH 2")).color(Color::BLACK)
                )
            }
            */
            column.into()
        }
    }
}
