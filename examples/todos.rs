use iced::{
    button, scrollable, text::HorizontalAlignment, text_input, Align,
    Application, Background, Button, Checkbox, Color, Column, Container,
    Element, Font, Length, Row, Scrollable, Text, TextInput,
};

pub fn main() {
    Todos::default().run()
}

#[derive(Debug, Default)]
struct Todos {
    scroll: scrollable::State,
    input: text_input::State,
    input_value: String,
    tasks: Vec<Task>,
}

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    CreateTask,
    TaskMessage(usize, TaskMessage),
}

impl Application for Todos {
    type Message = Message;

    fn title(&self) -> String {
        String::from("Todos - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::InputChanged(value) => {
                self.input_value = value;
            }
            Message::CreateTask => {
                if !self.input_value.is_empty() {
                    self.tasks.push(Task::new(self.input_value.clone()));
                    self.input_value.clear();
                }
            }
            Message::TaskMessage(i, TaskMessage::Delete) => {
                self.tasks.remove(i);
            }
            Message::TaskMessage(i, task_message) => {
                if let Some(task) = self.tasks.get_mut(i) {
                    task.update(task_message);
                }
            }
        }

        dbg!(self);
    }

    fn view(&mut self) -> Element<Message> {
        let title = Text::new("todos")
            .size(100)
            .color(GRAY)
            .horizontal_alignment(HorizontalAlignment::Center);

        let input = TextInput::new(
            &mut self.input,
            "What needs to be done?",
            &self.input_value,
            Message::InputChanged,
        )
        .padding(15)
        .size(30)
        .on_submit(Message::CreateTask);

        let tasks: Element<_> =
            if self.tasks.len() > 0 {
                self.tasks
                    .iter_mut()
                    .enumerate()
                    .fold(Column::new().spacing(20), |column, (i, task)| {
                        column.push(task.view().map(move |message| {
                            Message::TaskMessage(i, message)
                        }))
                    })
                    .into()
            } else {
                Container::new(
                    Text::new("You do not have any tasks! :D")
                        .size(25)
                        .horizontal_alignment(HorizontalAlignment::Center)
                        .color([0.7, 0.7, 0.7]),
                )
                .width(Length::Fill)
                .height(Length::Units(200))
                .center_y()
                .into()
            };

        let content = Column::new()
            .max_width(800)
            .spacing(20)
            .push(title)
            .push(input)
            .push(tasks);

        Scrollable::new(&mut self.scroll)
            .padding(40)
            .push(Container::new(content).width(Length::Fill).center_x())
            .into()
    }
}

#[derive(Debug)]
struct Task {
    description: String,
    completed: bool,
    state: TaskState,
}

#[derive(Debug)]
pub enum TaskState {
    Idle {
        edit_button: button::State,
    },
    Editing {
        text_input: text_input::State,
        delete_button: button::State,
    },
}

#[derive(Debug, Clone)]
pub enum TaskMessage {
    Completed(bool),
    Edit,
    DescriptionEdited(String),
    FinishEdition,
    Delete,
}

impl Task {
    fn new(description: String) -> Self {
        Task {
            description,
            completed: false,
            state: TaskState::Idle {
                edit_button: button::State::new(),
            },
        }
    }

    fn update(&mut self, message: TaskMessage) {
        match message {
            TaskMessage::Completed(completed) => {
                self.completed = completed;
            }
            TaskMessage::Edit => {
                self.state = TaskState::Editing {
                    text_input: text_input::State::focused(&self.description),
                    delete_button: button::State::new(),
                };
            }
            TaskMessage::DescriptionEdited(new_description) => {
                self.description = new_description;
            }
            TaskMessage::FinishEdition => {
                if !self.description.is_empty() {
                    self.state = TaskState::Idle {
                        edit_button: button::State::new(),
                    }
                }
            }
            TaskMessage::Delete => {}
        }
    }

    fn view(&mut self) -> Element<TaskMessage> {
        match &mut self.state {
            TaskState::Idle { edit_button } => {
                let checkbox = Checkbox::new(
                    self.completed,
                    &self.description,
                    TaskMessage::Completed,
                );

                Row::new()
                    .spacing(20)
                    .align_items(Align::Center)
                    .push(checkbox)
                    .push(
                        Button::new(
                            edit_button,
                            edit_icon().color([0.5, 0.5, 0.5]),
                        )
                        .on_press(TaskMessage::Edit)
                        .padding(10),
                    )
                    .into()
            }
            TaskState::Editing {
                text_input,
                delete_button,
            } => {
                let text_input = TextInput::new(
                    text_input,
                    "Describe your task...",
                    &self.description,
                    TaskMessage::DescriptionEdited,
                )
                .on_submit(TaskMessage::FinishEdition)
                .padding(10);

                Row::new()
                    .spacing(20)
                    .align_items(Align::Center)
                    .push(text_input)
                    .push(
                        Button::new(
                            delete_button,
                            Row::new()
                                .spacing(10)
                                .push(delete_icon().color(Color::WHITE))
                                .push(
                                    Text::new("Delete")
                                        .width(Length::Shrink)
                                        .color(Color::WHITE),
                                ),
                        )
                        .on_press(TaskMessage::Delete)
                        .padding(10)
                        .border_radius(5)
                        .background(Background::Color([0.8, 0.2, 0.2].into())),
                    )
                    .into()
            }
        }
    }
}

// Colors
const GRAY: Color = Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};

// Fonts
const ICONS: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("./resources/icons.ttf"),
};

fn icon(unicode: char) -> Text {
    Text::new(&unicode.to_string())
        .font(ICONS)
        .width(Length::Units(20))
        .horizontal_alignment(HorizontalAlignment::Center)
        .size(20)
}

fn edit_icon() -> Text {
    icon('\u{F303}')
}

fn delete_icon() -> Text {
    icon('\u{F1F8}')
}
