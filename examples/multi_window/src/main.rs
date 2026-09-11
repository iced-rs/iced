use iced::widget::{
    button, center, center_x, column, container, operation, scrollable, space, text, text_input,
};
use iced::window::{self, MonitorIndex, MonitorList};
use iced::{Center, Element, Fill, Function, Subscription, Task, Theme, Vector};

use std::collections::BTreeMap;

fn main() -> iced::Result {
    iced::daemon(Example::new, Example::update, Example::view)
        .subscription(Example::subscription)
        .title(Example::title)
        .theme(Example::theme)
        .scale_factor(Example::scale_factor)
        .run()
}

struct Example {
    windows: BTreeMap<window::Id, Window>,
}

#[derive(Debug)]
struct Window {
    title: String,
    scale_input: String,
    current_scale: f32,
    theme: Theme,
    monitors: Option<MonitorList>,
    selected_monitor: Option<MonitorIndex>,
}

#[derive(Debug, Clone)]
enum Message {
    OpenWindow(Option<MonitorIndex>),
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    ScaleInputChanged(window::Id, String),
    ScaleChanged(window::Id, String),
    TitleChanged(window::Id, String),
    ListMonitors(window::Id),
    MonitorsListed(window::Id, MonitorList),
    MonitorSelected(window::Id, MonitorIndex),
}

impl Example {
    fn new() -> (Self, Task<Message>) {
        let (_, open) = window::open(window::Settings::default());

        (
            Self {
                windows: BTreeMap::new(),
            },
            open.map(Message::WindowOpened),
        )
    }

    fn title(&self, window: window::Id) -> String {
        self.windows
            .get(&window)
            .map(|window| window.title.clone())
            .unwrap_or_default()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenWindow(monitor) => {
                let Some(last_window) = self.windows.keys().last() else {
                    return Task::none();
                };

                window::position(*last_window)
                    .then(move |last_position| {
                        let mut position = last_position.unwrap_or_default();
                        position.position = position.position + Vector::new(20.0, 20.0);
                        position.monitor_index = monitor.or(position.monitor_index);

                        let (_, open) = window::open(window::Settings {
                            position: position.into(),
                            ..window::Settings::default()
                        });

                        open
                    })
                    .map(Message::WindowOpened)
            }
            Message::WindowOpened(id) => {
                let window = Window::new(self.windows.len() + 1);
                let focus_input = operation::focus(format!("input-{id}"));

                self.windows.insert(id, window);

                focus_input
            }
            Message::WindowClosed(id) => {
                self.windows.remove(&id);

                if self.windows.is_empty() {
                    iced::exit()
                } else {
                    Task::none()
                }
            }
            Message::ScaleInputChanged(id, scale) => {
                if let Some(window) = self.windows.get_mut(&id) {
                    window.scale_input = scale;
                }

                Task::none()
            }
            Message::ScaleChanged(id, scale) => {
                if let Some(window) = self.windows.get_mut(&id) {
                    window.current_scale = scale
                        .parse()
                        .unwrap_or(window.current_scale)
                        .clamp(0.5, 5.0);
                }

                Task::none()
            }
            Message::TitleChanged(id, title) => {
                if let Some(window) = self.windows.get_mut(&id) {
                    window.title = title;
                }

                Task::none()
            }
            Message::ListMonitors(id) => {
                if let Some(window) = self.windows.get_mut(&id) {
                    window.monitors = None;
                    window::list_monitors().map(Message::MonitorsListed.with(id))
                } else {
                    Task::none()
                }
            }
            Message::MonitorsListed(id, monitors) => {
                if let Some(window) = self.windows.get_mut(&id) {
                    window.monitors = Some(monitors);
                }

                Task::none()
            }
            Message::MonitorSelected(id, index) => {
                if let Some(window) = self.windows.get_mut(&id) {
                    window.selected_monitor = Some(index);
                }

                Task::none()
            }
        }
    }

    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if let Some(window) = self.windows.get(&window_id) {
            center(window.view(window_id)).into()
        } else {
            space().into()
        }
    }

    fn theme(&self, window: window::Id) -> Option<Theme> {
        Some(self.windows.get(&window)?.theme.clone())
    }

    fn scale_factor(&self, window: window::Id) -> f32 {
        self.windows
            .get(&window)
            .map(|window| window.current_scale)
            .unwrap_or(1.0)
    }

    fn subscription(&self) -> Subscription<Message> {
        window::close_events().map(Message::WindowClosed)
    }
}

impl Window {
    fn new(count: usize) -> Self {
        Self {
            title: format!("Window_{count}"),
            scale_input: "1.0".to_string(),
            current_scale: 1.0,
            theme: Theme::ALL[count % Theme::ALL.len()].clone(),
            monitors: None,
            selected_monitor: None,
        }
    }

    fn view(&self, id: window::Id) -> Element<'_, Message> {
        let scale_input = column![
            text("Window scale factor:"),
            text_input("Window Scale", &self.scale_input)
                .on_input(Message::ScaleInputChanged.with(id))
                .on_submit(Message::ScaleChanged(id, self.scale_input.to_string()))
        ];

        let title_input = column![
            text("Window title:"),
            text_input("Window Title", &self.title)
                .on_input(Message::TitleChanged.with(id))
                .id(format!("input-{id}"))
        ];

        let monitor_select = if let Some(list) = &self.monitors {
            let column = column(list.iter().map(|monitor| {
                button(text(
                    monitor.name().unwrap_or("Unknown Monitor").to_string(),
                ))
                .on_press_maybe(if self.selected_monitor != Some(monitor.index()) {
                    Some(Message::MonitorSelected(id, monitor.index()))
                } else {
                    None
                })
                .into()
            }))
            .push(button("Refresh").on_press(Message::ListMonitors(id)))
            .spacing(10);

            Element::new(column)
        } else {
            button("List monitors")
                .on_press(Message::ListMonitors(id))
                .into()
        };

        let new_window_button =
            button(text("New Window")).on_press(Message::OpenWindow(self.selected_monitor));

        let content = column![scale_input, title_input, monitor_select, new_window_button]
            .spacing(50)
            .width(Fill)
            .align_x(Center)
            .width(200);

        container(scrollable(center_x(content))).padding(10).into()
    }
}
