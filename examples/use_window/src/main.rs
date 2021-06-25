use iced::{button, text_input, executor, Align, Button, Column, Element, Application, Settings, Text, Window, Clipboard, TextInput, Command, runtime};

pub fn main() -> iced::Result {
    App::run(Settings::default())
}

#[derive(Default)]
struct App {
    height: String,
    width: String,
    x: String,
    y: String,
    height_input: text_input::State,
    width_input: text_input::State,
    x_input: text_input::State,
    y_input: text_input::State,
    resize_button: button::State,
    move_button: button::State
}

#[derive(Debug, Clone)]
enum Message {
    ResizePressed,
    MovePressed,
    HeightChanged(String),
    WidthChanged(String),
    XChanged(String),
    YChanged(String),
}

impl Application for App {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Use Window - Iced")
    }

    fn update(&mut self, message: Message, _: &mut Clipboard, window: &Window) -> Command<Self::Message> {
        match message {
            Message::ResizePressed => {
                window.set_inner_size(runtime::winit::dpi::LogicalSize::new(
                    self.width.parse::<u32>().unwrap(), 
                    self.height.parse::<u32>().unwrap()
                ));
            },
            Message::MovePressed => {
                window.set_outer_position(runtime::winit::dpi::LogicalPosition::new(
                    self.x.parse::<i32>().unwrap(), 
                    self.y.parse::<i32>().unwrap()
                ));
            },
            Message::HeightChanged(height) => {
                self.height = height;
            },
            Message::WidthChanged(width) => {
                self.width = width;
            }
            Message::XChanged(x) => {
                self.x = x;
            },
            Message::YChanged(y) => {
                self.y = y;
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        Column::new()
            .padding(20)
            .align_items(Align::Center)
            .push(TextInput::new(&mut self.width_input, "Width...", &self.width, Message::WidthChanged))
            .push(TextInput::new(&mut self.height_input, "Height...", &self.height, Message::HeightChanged))
            .push(
                Button::new(&mut self.resize_button, Text::new("Resize"))
                    .on_press(Message::ResizePressed),
            )
            .push(TextInput::new(&mut self.x_input, "X...", &self.x, Message::XChanged))
            .push(TextInput::new(&mut self.y_input, "Y...", &self.y, Message::YChanged))
            .push(
                Button::new(&mut self.move_button, Text::new("Move"))
                    .on_press(Message::MovePressed),
            )
            .into()
    }
}
