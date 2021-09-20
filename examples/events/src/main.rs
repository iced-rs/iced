use iced::{
    alignment, button, executor, Alignment, Application, Button, Checkbox,
    Column, Command, Container, Element, Length, Settings, Subscription, Text,
};
use iced_native::{window, Event};

pub fn main() -> iced::Result {
    Events::run(Settings {
        exit_on_close_request: false,
        ..Settings::default()
    })
}

#[derive(Debug, Default)]
struct Events {
    last: Vec<iced_native::Event>,
    enabled: bool,
    exit: button::State,
    should_exit: bool,
}

#[derive(Debug, Clone)]
enum Message {
    EventOccurred(iced_native::Event),
    Toggled(bool),
    Exit,
}

impl Application for Events {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Events, Command<Message>) {
        (Events::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Events - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::EventOccurred(event) if self.enabled => {
                self.last.push(event);

                if self.last.len() > 5 {
                    let _ = self.last.remove(0);
                }
            }
            Message::EventOccurred(event) => {
                if let Event::Window(window::Event::CloseRequested) = event {
                    self.should_exit = true;
                }
            }
            Message::Toggled(enabled) => {
                self.enabled = enabled;
            }
            Message::Exit => {
                self.should_exit = true;
            }
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_native::subscription::events().map(Message::EventOccurred)
    }

    fn should_exit(&self) -> bool {
        self.should_exit
    }

    fn view(&mut self) -> Element<Message> {
        let events = self.last.iter().fold(
            Column::new().spacing(10),
            |column, event| {
                column.push(Text::new(format!("{:?}", event)).size(40))
            },
        );

        let toggle = Checkbox::new(
            self.enabled,
            "Listen to runtime events",
            Message::Toggled,
        );

        let exit = Button::new(
            &mut self.exit,
            Text::new("Exit")
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Center),
        )
        .width(Length::Units(100))
        .padding(10)
        .on_press(Message::Exit);

        let content = Column::new()
            .align_items(Alignment::Center)
            .spacing(20)
            .push(events)
            .push(toggle)
            .push(exit);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
