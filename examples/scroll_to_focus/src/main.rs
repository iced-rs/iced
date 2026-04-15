use iced::advanced::widget::operation::{Focusable, Operation, Outcome};
use iced::widget::operation::EnsureVisibleConfig;
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input};
use iced::{Center, Element, Fill, Rectangle, Task};

use iced::widget::Id;

const ITEM_COUNT: usize = 50;

pub fn main() -> iced::Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    iced::application(App::default, App::update, App::view)
        .title("Scroll to Focus")
        .run()
}

struct App {
    index_input: String,
    focused_index: Option<usize>,
    animate: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            index_input: String::new(),
            focused_index: None,
            animate: true,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    IndexInputChanged(String),
    GoToIndex,
    ItemPressed(usize),
    ToggleAnimate(bool),
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::IndexInputChanged(value) => {
                self.index_input = value;
                Task::none()
            }
            Message::GoToIndex => {
                let Ok(index) = self.index_input.parse::<usize>() else {
                    return Task::none();
                };
                if index >= ITEM_COUNT {
                    return Task::none();
                }
                log::info!("Focusing item {index}");
                focus_at(index)
            }
            Message::ItemPressed(index) => {
                log::info!("Item {index} pressed");
                self.focused_index = Some(index);
                Task::none()
            }
            Message::ToggleAnimate(value) => {
                self.animate = value;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let items: Vec<Element<Message>> = (0..ITEM_COUNT)
            .map(|i| {
                button(text(format!("Item {i}")).width(Fill).center())
                    .on_press(Message::ItemPressed(i))
                    .width(Fill)
                    .into()
            })
            .collect();

        let config = if self.animate {
            EnsureVisibleConfig::default()
        } else {
            EnsureVisibleConfig::default().instant()
        };

        let list = scrollable(column(items).spacing(4).padding(8))
            .ensure_focused_visible(config)
            .width(300)
            .height(Fill);

        let index_input = text_input("Item index (0–49)", &self.index_input)
            .on_input(Message::IndexInputChanged)
            .on_submit(Message::GoToIndex)
            .width(200);

        let go_button = button(text("Go")).on_press(Message::GoToIndex);

        let animate_toggle = checkbox(self.animate)
            .label("Animate scroll")
            .on_toggle(Message::ToggleAnimate);

        let status = if let Some(i) = self.focused_index {
            text(format!("Last pressed: Item {i}"))
        } else {
            text("Use Tab/Arrows or type an index")
        };

        let controls = column![index_input, go_button, animate_toggle, status]
            .spacing(8)
            .padding(16)
            .width(Fill)
            .align_x(Center);

        container(row![list, controls].spacing(16).padding(16))
            .center(Fill)
            .into()
    }
}

/// Focus the Nth focusable widget in the tree (0-indexed).
fn focus_at<T: Send + 'static>(target: usize) -> Task<T> {
    struct FocusAt {
        target: usize,
        current: usize,
    }

    impl<T> Operation<T> for FocusAt {
        fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if self.current == self.target {
                state.focus();
                iced::advanced::widget::operation::focusable::mark_focus_dirty();
            } else {
                state.unfocus();
            }
            self.current += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }

        fn finish(&self) -> Outcome<T> {
            Outcome::None
        }
    }

    iced::task::widget(FocusAt { target, current: 0 })
}
