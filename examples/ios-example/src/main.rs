use iced::{
    //button, scrollable, slider, text_input, Button, Checkbox, Color, Column,
    //Container, Element, HorizontalAlignment, Image, Length, Radio, Row,
    //Sandbox, //Scrollable, Settings, Slider, Space, Text, TextInput,
    Application,
    Settings,
    Command,
    executor,
    Element,
    Text,
    Container,
    Checkbox,
};
pub fn main() {
    env_logger::init();

    Simple::run(Settings::default())
}

#[derive(Debug, Default)]
pub struct Simple {
    enabled: bool,
}
#[derive(Debug, Clone)]
pub enum Message {
    //EventOccurred(iced_native::Event),
    Toggled(bool),
}

impl Application for Simple {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Events - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        /*
        let toggle = Checkbox::new(
            self.enabled,
            "Listen to runtime events",
            Message::Toggled,
        );
        toggle.into()
        */
        let text = Text::new("foobar");
        text.into()
    }
}
