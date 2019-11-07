use iced::{
    scrollable, text::HorizontalAlignment, text_input, Align, Application,
    Checkbox, Color, Column, Element, Length, Scrollable, Text, TextInput,
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
    TaskChanged(usize, bool),
}

impl Application for Todos {
    type Message = Message;

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
            Message::TaskChanged(i, completed) => {
                if let Some(task) = self.tasks.get_mut(i) {
                    task.completed = completed;
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

        let tasks = self.tasks.iter_mut().enumerate().fold(
            Column::new().spacing(20),
            |column, (i, task)| {
                column.push(
                    task.view()
                        .map(move |state| Message::TaskChanged(i, state)),
                )
            },
        );

        let content = Column::new()
            .max_width(Length::Units(800))
            .align_self(Align::Center)
            .spacing(20)
            .push(title)
            .push(input)
            .push(tasks);

        Scrollable::new(&mut self.scroll)
            .padding(40)
            .push(content)
            .into()
    }
}

#[derive(Debug)]
struct Task {
    description: String,
    completed: bool,
}

impl Task {
    fn new(description: String) -> Self {
        Task {
            description,
            completed: false,
        }
    }

    fn view(&mut self) -> Element<bool> {
        Checkbox::new(self.completed, &self.description, |checked| checked)
            .into()
    }
}

// Colors
const GRAY: Color = Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};
