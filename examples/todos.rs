use iced::{
    button, scrollable, text_input, Align, Application, Background, Button,
    Checkbox, Color, Column, Command, Container, Element, Font,
    HorizontalAlignment, Length, Palette, Row, Scrollable, Settings, Text,
    TextInput, TextStyle,
};
use serde::{Deserialize, Serialize};

pub fn main() {
    Todos::run(Settings::default())
}

#[derive(Debug)]
enum Todos {
    Loading,
    Loaded(State),
}

#[derive(Debug, Default)]
struct State {
    scroll: scrollable::State,
    input: text_input::State,
    input_value: String,
    filter: Filter,
    tasks: Vec<Task>,
    controls: Controls,
    dirty: bool,
    saving: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Loaded(Result<SavedState, LoadError>),
    Saved(Result<(), SaveError>),
    InputChanged(String),
    CreateTask,
    FilterChanged(Filter),
    TaskMessage(usize, TaskMessage),
}

impl Application for Todos {
    type Message = Message;

    fn new() -> (Todos, Command<Message>) {
        (
            Todos::Loading,
            Command::perform(SavedState::load(), Message::Loaded),
        )
    }

    fn title(&self) -> String {
        let dirty = match self {
            Todos::Loading => false,
            Todos::Loaded(state) => state.dirty,
        };

        format!("Todos{} - Iced", if dirty { "*" } else { "" })
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match self {
            Todos::Loading => {
                match message {
                    Message::Loaded(Ok(state)) => {
                        *self = Todos::Loaded(State {
                            input_value: state.input_value,
                            filter: state.filter,
                            tasks: state.tasks,
                            ..State::default()
                        });
                    }
                    Message::Loaded(Err(_)) => {
                        *self = Todos::Loaded(State::default());
                    }
                    _ => {}
                }

                Command::none()
            }
            Todos::Loaded(state) => {
                let mut saved = false;

                match message {
                    Message::InputChanged(value) => {
                        state.input_value = value;
                    }
                    Message::CreateTask => {
                        if !state.input_value.is_empty() {
                            state
                                .tasks
                                .push(Task::new(state.input_value.clone()));
                            state.input_value.clear();
                        }
                    }
                    Message::FilterChanged(filter) => {
                        state.filter = filter;
                    }
                    Message::TaskMessage(i, TaskMessage::Delete) => {
                        state.tasks.remove(i);
                    }
                    Message::TaskMessage(i, task_message) => {
                        if let Some(task) = state.tasks.get_mut(i) {
                            task.update(task_message);
                        }
                    }
                    Message::Saved(_) => {
                        state.saving = false;
                        saved = true;
                    }
                    _ => {}
                }

                if !saved {
                    state.dirty = true;
                }

                if state.dirty && !state.saving {
                    state.dirty = false;
                    state.saving = true;

                    Command::perform(
                        SavedState {
                            input_value: state.input_value.clone(),
                            filter: state.filter,
                            tasks: state.tasks.clone(),
                        }
                        .save(),
                        Message::Saved,
                    )
                } else {
                    Command::none()
                }
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        match self {
            Todos::Loading => loading_message(),
            Todos::Loaded(State {
                scroll,
                input,
                input_value,
                filter,
                tasks,
                controls,
                ..
            }) => {
                let title = Text::new("todos")
                    .size(100)
                    .horizontal_alignment(HorizontalAlignment::Center);

                let input = TextInput::new(
                    input,
                    "What needs to be done?",
                    input_value,
                    Message::InputChanged,
                )
                .padding(15)
                .size(30)
                .on_submit(Message::CreateTask);

                let controls = controls.view(&tasks, *filter);
                let filtered_tasks =
                    tasks.iter().filter(|task| filter.matches(task));

                let tasks: Element<_> = if filtered_tasks.count() > 0 {
                    tasks
                        .iter_mut()
                        .enumerate()
                        .filter(|(_, task)| filter.matches(task))
                        .fold(Column::new().spacing(20), |column, (i, task)| {
                            column.push(task.view().map(move |message| {
                                Message::TaskMessage(i, message)
                            }))
                        })
                        .into()
                } else {
                    empty_message(match filter {
                        Filter::All => "You have not created a task yet...",
                        Filter::Active => "All your tasks are done! :D",
                        Filter::Completed => {
                            "You have not completed a task yet..."
                        }
                    })
                };

                let content = Column::new()
                    .max_width(800)
                    .spacing(20)
                    .push(title)
                    .push(input)
                    .push(controls)
                    .push(tasks);

                Scrollable::new(scroll)
                    .padding(40)
                    .push(
                        Container::new(content).width(Length::Fill).center_x(),
                    )
                    .into()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    description: String,
    completed: bool,

    #[serde(skip)]
    state: TaskState,
}

#[derive(Debug, Clone)]
pub enum TaskState {
    Idle {
        edit_button: button::State,
    },
    Editing {
        text_input: text_input::State,
        delete_button: button::State,
    },
}

impl Default for TaskState {
    fn default() -> Self {
        TaskState::Idle {
            edit_button: button::State::new(),
        }
    }
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
                    text_input: text_input::State::focused(),
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
                        Button::new(edit_button, edit_icon())
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

                let palette = Palette::default();

                Row::new()
                    .spacing(20)
                    .align_items(Align::Center)
                    .push(text_input)
                    .push(
                        Button::new(
                            delete_button,
                            Row::new()
                                .spacing(10)
                                .push(delete_icon().change_style(|style| {
                                    style.text_color = Color::WHITE
                                }))
                                .push(
                                    Text::new_with_style(
                                        "Delete",
                                        TextStyle {
                                            text_color: Color::WHITE,
                                        },
                                    )
                                    .width(Length::Shrink),
                                ),
                        )
                        .on_press(TaskMessage::Delete)
                        .padding(10)
                        .change_style(|style| {
                            style.border_radius = 5;
                            style.background =
                                Some(Background::Color(palette.negative));
                            style.border_color = palette.negative;
                        }),
                    )
                    .into()
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Controls {
    all_button: button::State,
    active_button: button::State,
    completed_button: button::State,
}

impl Controls {
    fn view(&mut self, tasks: &[Task], current_filter: Filter) -> Row<Message> {
        let palette = Palette::default();

        let Controls {
            all_button,
            active_button,
            completed_button,
        } = self;

        let tasks_left = tasks.iter().filter(|task| !task.completed).count();

        let filter_button = |state, label, filter, current_filter| {
            let label = Text::new(label).size(16).width(Length::Shrink);
            let button = if filter == current_filter {
                Button::new(state, label).change_style(|style| {
                    style.border_radius = 10;
                    style.background =
                        Some(Background::Color(palette.highlight));
                })
            } else {
                Button::new(state, label)
                    .change_style(|style| style.border_radius = 10)
            };

            button.on_press(Message::FilterChanged(filter)).padding(8)
        };

        Row::new()
            .spacing(20)
            .align_items(Align::Center)
            .push(
                Text::new(&format!(
                    "{} {} left",
                    tasks_left,
                    if tasks_left == 1 { "task" } else { "tasks" }
                ))
                .size(16),
            )
            .push(
                Row::new()
                    .width(Length::Shrink)
                    .spacing(10)
                    .push(filter_button(
                        all_button,
                        "All",
                        Filter::All,
                        current_filter,
                    ))
                    .push(filter_button(
                        active_button,
                        "Active",
                        Filter::Active,
                        current_filter,
                    ))
                    .push(filter_button(
                        completed_button,
                        "Completed",
                        Filter::Completed,
                        current_filter,
                    )),
            )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Filter {
    All,
    Active,
    Completed,
}

impl Default for Filter {
    fn default() -> Self {
        Filter::All
    }
}

impl Filter {
    fn matches(&self, task: &Task) -> bool {
        match self {
            Filter::All => true,
            Filter::Active => !task.completed,
            Filter::Completed => task.completed,
        }
    }
}

fn loading_message() -> Element<'static, Message> {
    Container::new(
        Text::new("Loading...")
            .horizontal_alignment(HorizontalAlignment::Center)
            .size(50),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_y()
    .into()
}

fn empty_message(message: &str) -> Element<'static, Message> {
    Container::new(
        Text::new(message)
            .size(25)
            .horizontal_alignment(HorizontalAlignment::Center)
            .change_style(|style: &mut TextStyle| {
                style.text_color = [0.7, 0.7, 0.7].into()
            }),
    )
    .width(Length::Fill)
    .height(Length::Units(200))
    .center_y()
    .into()
}

// Fonts
const ICONS: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("resources/icons.ttf"),
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

// Persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SavedState {
    input_value: String,
    filter: Filter,
    tasks: Vec<Task>,
}

#[derive(Debug, Clone)]
enum LoadError {
    FileError,
    FormatError,
}

#[derive(Debug, Clone)]
enum SaveError {
    DirectoryError,
    FileError,
    WriteError,
    FormatError,
}

impl SavedState {
    fn path() -> std::path::PathBuf {
        let mut path = if let Some(project_dirs) =
            directories::ProjectDirs::from("rs", "Iced", "Todos")
        {
            project_dirs.data_dir().into()
        } else {
            std::env::current_dir().unwrap_or(std::path::PathBuf::new())
        };

        path.push("todos.json");

        path
    }

    async fn load() -> Result<SavedState, LoadError> {
        use std::io::Read;

        let mut contents = String::new();

        let mut file = std::fs::File::open(Self::path())
            .map_err(|_| LoadError::FileError)?;

        file.read_to_string(&mut contents)
            .map_err(|_| LoadError::FileError)?;

        serde_json::from_str(&contents).map_err(|_| LoadError::FormatError)
    }

    async fn save(self) -> Result<(), SaveError> {
        use std::io::Write;

        let json = serde_json::to_string_pretty(&self)
            .map_err(|_| SaveError::FormatError)?;

        let path = Self::path();

        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)
                .map_err(|_| SaveError::DirectoryError)?;
        }

        let mut file =
            std::fs::File::create(path).map_err(|_| SaveError::FileError)?;

        file.write_all(json.as_bytes())
            .map_err(|_| SaveError::WriteError)?;

        // This is a simple way to save at most once every couple seconds
        // We will be able to get rid of it once we implement event
        // subscriptions
        std::thread::sleep(std::time::Duration::from_secs(2));

        Ok(())
    }
}
