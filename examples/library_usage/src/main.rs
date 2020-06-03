use iced::{
    executor, Align, Application, Column, Command, Container, Element, Length,
    Settings, Subscription, Text,
};
use winit::event_loop::EventLoop;

pub fn main() {
    let mut event_loop = EventLoop::with_user_event();
    let mut context = Events::initialize(&mut event_loop, Settings::default());

    event_loop.run(move |event, _, control_flow| {
        context.handle_winit_event(event, control_flow);
    })
}

#[derive(Debug, Default)]
struct Events {
    last: Option<iced_native::Event>,
}

#[derive(Debug, Clone)]
struct Message(iced_native::Event);

impl Application for Events {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Events, Command<Message>) {
        (Events::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Library Usage - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        self.last = Some(message.0);

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_native::subscription::events().map(Message)
    }

    fn view(&mut self) -> Element<Message> {
        let event = self.last.iter().fold(
            Column::new().spacing(10),
            |column, event| {
                column.push(Text::new(format!("{:#?}", event)).size(20))
            },
        );

        let content = Column::new().align_items(Align::Center).push(event);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
