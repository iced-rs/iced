#[macro_use] extern crate log;
use iced::{
    //button, scrollable, slider, text_input, Button, Checkbox, Color,
    Column,
    //Container, Element, HorizontalAlignment, Image, Length, Radio, Row,
    //Sandbox, //Scrollable, Settings, Slider, Space, Text, TextInput,
    Sandbox,
    Settings,
    Command,
    executor,
    Element,
    Text,
    TextInput,
    text_input,
    Container,
    Color,
    Checkbox,
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
    text_state: text_input::State,
}
#[derive(Debug, Clone)]
pub enum Message {
    //EventOccurred(iced_native::Event),
    Toggled(bool),
    TextUpdated(String),
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
        debug!("GOT NEW MESSAGE: {:?}", message);
        match message {
            Message::TextUpdated(val) => {
                self.text = val;
            },
            _ => {
            },
        }
    }

    fn view(&mut self) -> Element<Message> {
        debug!("RERUNNING VIEW : {:#?}", self);
        let column = Column::new()
            .push(
                TextInput::new(
                    &mut self.text_state,
                    "",
                    "",
                    |s| { Message::TextUpdated(s) }
                )
            )
            .push(
                Text::new(&self.text).color(Color::BLACK)
            )
            .push(
                Text::new(String::from("foo foo foo")).color(Color::BLACK)
            )
            ;
        column.into()

    }
}
