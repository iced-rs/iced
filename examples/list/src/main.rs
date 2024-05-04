use iced::widget::{
    button, center, column, container, horizontal_space, list, row, scrollable,
    text,
};
use iced::{Alignment, Element, Length, Theme};

pub fn main() -> iced::Result {
    iced::program("List - Iced", List::update, List::view)
        .theme(|_| Theme::TokyoNight)
        .run()
}

struct List {
    content: list::Content<(usize, State)>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Update(usize),
    Remove(usize),
}

impl List {
    fn update(&mut self, message: Message) {
        match message {
            Message::Update(index) => {
                if let Some((_id, state)) = self.content.get_mut(index) {
                    *state = State::Updated;
                }
            }
            Message::Remove(index) => {
                let _ = self.content.remove(index);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        center(
            scrollable(
                container(list(&self.content, |index, (id, state)| {
                    row![
                        match state {
                            State::Idle =>
                                Element::from(text(format!("I am item {id}!"))),
                            State::Updated => center(
                                column![
                                    text(format!("I am item {id}!")),
                                    text("... but different!")
                                ]
                                .spacing(20)
                            )
                            .height(300)
                            .into(),
                        },
                        horizontal_space(),
                        button("Update").on_press_maybe(
                            matches!(state, State::Idle)
                                .then_some(Message::Update(index))
                        ),
                        button("Remove")
                            .on_press(Message::Remove(index))
                            .style(button::danger)
                    ]
                    .spacing(10)
                    .padding(5)
                    .align_items(Alignment::Center)
                    .into()
                }))
                .padding(10),
            )
            .width(Length::Fill),
        )
        .padding(10)
        .into()
    }
}

impl Default for List {
    fn default() -> Self {
        Self {
            content: list::Content::from_iter(
                (0..1_000).map(|id| (id, State::Idle)),
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Idle,
    Updated,
}
