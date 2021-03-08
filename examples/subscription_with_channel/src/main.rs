use iced::{
    Align, Application, Button, button, Column, Command, Container, Element, executor, Length,
    Row, Settings, Subscription, Text,
};
use tokio::sync::mpsc::Sender;

fn main() -> iced::Result {
    Example::run(
        Settings::default()
    )
}

#[derive(Debug)]
struct Example {
    numbers: Vec<u32>,
    button_add: button::State,
    button_clear: button::State,
    button_start_over: button::State,
    send: Option<Sender<my_stream::SubMsg>>,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Init(Sender<my_stream::SubMsg>, u32),
    NextNum,
    RecvNum(u32),
    ClearView,
    RestartCounter,
    DoNothing,
}

impl Application for Example {
    type Executor = executor::Default;
    type Message = Msg;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                numbers: vec![],
                button_add: button::State::new(),
                button_clear: button::State::new(),
                button_start_over: button::State::new(),
                send: None,
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        "Iced Example: Subscriptions + Tokio Channels".to_string()
    }

    fn update(&mut self, message: Msg) -> Command<Self::Message> {
        match message {
            Msg::Init(send, counter) => {
                self.send = Some(send);
                self.numbers.push(counter);
                Command::none()
            }
            Msg::NextNum => {
                if let Some(send) = self.send.as_ref() {
                    let send_cp = send.clone();
                    Command::perform(
                        async move { send_cp.send(my_stream::SubMsg::GetValue).await },
                        move |_| { Msg::DoNothing },
                    )
                } else {
                    Command::none()
                }
            }
            Msg::RecvNum(num) => {
                self.numbers.push(num);
                Command::none()
            }
            Msg::DoNothing => {
                Command::none()
            }
            Msg::ClearView => {
                self.numbers.clear();
                Command::none()
            }
            Msg::RestartCounter => {
                if let Some(send) = self.send.as_ref() {
                    let send_cp = send.clone();
                    Command::perform(
                        async move { send_cp.send(my_stream::SubMsg::RestartCounter).await },
                        move |_| { Msg::DoNothing },
                    )
                } else {
                    Command::none()
                }
            }
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // The key point about a subscription is that it is sent only when the stream returns.
        // The stream awaits the receiver, therefore this function will not spam it with requests
        // until it's done with the previous request, i.e. one request at a time
        iced::Subscription::from_recipe(my_stream::Sub { counter: 0 })
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let button_add_num = Button::new(&mut self.button_add, Text::new("Get more numbers"))
            .on_press(Msg::NextNum);

        let button_clear = Button::new(
            &mut self.button_clear,
            Text::new("Clear the view"),
        )
            .on_press(Msg::ClearView);

        let button_restart = Button::new(
            &mut self.button_start_over,
            Text::new("Restart the counter"),
        )
            .on_press(Msg::RestartCounter);

        let button_row = Row::new()
            .push(button_add_num)
            .push(button_clear)
            .push(button_restart)
            .padding(20)
            .align_items(Align::Center)
            .width(Length::Fill);

        let numbers = self.numbers.iter_mut()
            .fold(Column::new().spacing(5), |column, number| {
                column.push(
                    Text::new(format!("{}", number)).size(20)
                )
            })
            .align_items(Align::Start)
            .width(Length::Fill)
            .height(Length::Fill);

        let col = Column::new()
            .push(button_row)
            .push(numbers)
            .align_items(Align::Start);

        Container::new(col)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .padding(20)
            .into()
    }
}

mod my_stream {
    use std::hash::{Hash, Hasher};

    use iced_futures::futures;
    use tokio::sync::mpsc::channel;

    use crate::Msg;

    enum SubState {
        Init,
        Busy,
    }

    pub enum SubMsg {
        GetValue,
        RestartCounter,
    }

    pub struct Sub {
        pub counter: u32,
    }

    impl<H, I> iced_native::subscription::Recipe<H, I> for Sub
        where H: Hasher,
    {
        type Output = Msg;

        fn hash(&self, state: &mut H) {
            struct Marker;
            std::any::TypeId::of::<Marker>().hash(state);
            self.counter.hash(state);
        }

        fn stream(self: Box<Self>,
                  _input: futures::stream::BoxStream<'static, I>, )
                  -> futures::stream::BoxStream<'static, Self::Output>
        {
            Box::pin(futures::stream::unfold(
                (channel::<SubMsg>(1024), SubState::Init, self.counter),
                move |state| async move {
                    let ((send, mut recv), state, counter) = state;

                    match state {
                        SubState::Init => {
                            Some((
                                Msg::Init(send.clone(), counter),
                                ((send, recv), SubState::Busy, counter + 1)
                            ))
                        }
                        SubState::Busy => {
                            // this will keep the stream occupied until the next message from a sender
                            // arrives
                            let val = recv.recv().await;

                            // only after a message is received, an appropriate action is taken
                            match val.unwrap() {
                                SubMsg::GetValue => {
                                    Some((
                                        Msg::RecvNum(counter),
                                        ((send, recv), SubState::Busy, counter + 1)
                                    ))
                                }
                                SubMsg::RestartCounter => {
                                    Some((
                                        Msg::DoNothing,
                                        ((send, recv), SubState::Busy, 0)
                                    ))
                                }
                            }
                        }
                    }
                },
            ))
        }
    }
}