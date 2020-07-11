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
    //Container,
    Color,
    //Checkbox,
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
    text_state2: text_input::State,
}
#[derive(Debug, Clone)]
pub enum Message {
    //EventOccurred(iced_native::Event),
    Toggled(bool),
    TextUpdated(String),
    TextSubmit,
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
                Column::new()
                //.push(
                //    Text::new(String::from("FIRST FIRST")).color(Color::BLACK)
                //)
                .push(
                    TextInput::new(
                        &mut self.text_state,
                        "",
                        "",
                        |s| {
                            debug!("The 1st text box has \"{}\" in it!", s);
                            Message::TextUpdated(s)
                        }
                    ).on_submit(Message::TextSubmit),
                )
            )
            .push(
                Text::new(String::from("SECOND SECOND")).color(Color::BLACK)
            )
            .push(
                TextInput::new(
                    &mut self.text_state2,
                    "",
                    "",
                    |s| {
                        debug!("The 2nd text box has \"{}\" in it!", s);
                        Message::TextUpdated(s)
                    }
                )
            )
            //.push(
            //    Text::new(&self.text).color(Color::BLACK)
            //)
            ;
        column.into()

    }
}
