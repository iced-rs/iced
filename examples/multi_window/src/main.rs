use iced::alignment::{self, Alignment};
use iced::executor;
use iced::keyboard;
use iced::multi_window::Application;
use iced::theme::{self, Theme};
use iced::widget::pane_grid::{self, PaneGrid};
use iced::widget::{
    button, column, container, pick_list, row, scrollable, text, text_input,
};
use iced::window;
use iced::{Color, Command, Element, Length, Settings, Size, Subscription};
use iced_lazy::responsive;
use iced_native::{event, subscription, Event};

use std::collections::HashMap;

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

struct Example {
    windows: HashMap<window::Id, Window>,
    panes_created: usize,
    _focused: window::Id,
}

struct Window {
    title: String,
    panes: pane_grid::State<Pane>,
    focus: Option<pane_grid::Pane>,
}

#[derive(Debug, Clone)]
enum Message {
    Window(window::Id, WindowMessage),
}

#[derive(Debug, Clone)]
enum WindowMessage {
    Split(pane_grid::Axis, pane_grid::Pane),
    SplitFocused(pane_grid::Axis),
    FocusAdjacent(pane_grid::Direction),
    Clicked(pane_grid::Pane),
    Dragged(pane_grid::DragEvent),
    PopOut(pane_grid::Pane),
    Resized(pane_grid::ResizeEvent),
    TitleChanged(String),
    ToggleMoving(pane_grid::Pane),
    TogglePin(pane_grid::Pane),
    Close(pane_grid::Pane),
    CloseFocused,
    SelectedWindow(pane_grid::Pane, SelectableWindow),
    CloseWindow,
}

impl Application for Example {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let (panes, _) =
            pane_grid::State::new(Pane::new(0, pane_grid::Axis::Horizontal));
        let window = Window {
            panes,
            focus: None,
            title: String::from("Default window"),
        };

        (
            Example {
                windows: HashMap::from([(window::Id::MAIN, window)]),
                panes_created: 1,
                _focused: window::Id::MAIN,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Multi windowed pane grid - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        let Message::Window(id, message) = message;
        match message {
            WindowMessage::Split(axis, pane) => {
                let window = self.windows.get_mut(&id).unwrap();
                let result = window.panes.split(
                    axis,
                    &pane,
                    Pane::new(self.panes_created, axis),
                );

                if let Some((pane, _)) = result {
                    window.focus = Some(pane);
                }

                self.panes_created += 1;
            }
            WindowMessage::SplitFocused(axis) => {
                let window = self.windows.get_mut(&id).unwrap();
                if let Some(pane) = window.focus {
                    let result = window.panes.split(
                        axis,
                        &pane,
                        Pane::new(self.panes_created, axis),
                    );

                    if let Some((pane, _)) = result {
                        window.focus = Some(pane);
                    }

                    self.panes_created += 1;
                }
            }
            WindowMessage::FocusAdjacent(direction) => {
                let window = self.windows.get_mut(&id).unwrap();
                if let Some(pane) = window.focus {
                    if let Some(adjacent) =
                        window.panes.adjacent(&pane, direction)
                    {
                        window.focus = Some(adjacent);
                    }
                }
            }
            WindowMessage::Clicked(pane) => {
                let window = self.windows.get_mut(&id).unwrap();
                window.focus = Some(pane);
            }
            WindowMessage::CloseWindow => {
                let _ = self.windows.remove(&id);
                return window::close(id);
            }
            WindowMessage::Resized(pane_grid::ResizeEvent { split, ratio }) => {
                let window = self.windows.get_mut(&id).unwrap();
                window.panes.resize(&split, ratio);
            }
            WindowMessage::SelectedWindow(pane, selected) => {
                let window = self.windows.get_mut(&id).unwrap();
                let (mut pane, _) = window.panes.close(&pane).unwrap();
                pane.is_moving = false;

                if let Some(window) = self.windows.get_mut(&selected.0) {
                    let (&first_pane, _) = window.panes.iter().next().unwrap();
                    let result =
                        window.panes.split(pane.axis, &first_pane, pane);

                    if let Some((pane, _)) = result {
                        window.focus = Some(pane);
                    }
                }
            }
            WindowMessage::ToggleMoving(pane) => {
                let window = self.windows.get_mut(&id).unwrap();
                if let Some(pane) = window.panes.get_mut(&pane) {
                    pane.is_moving = !pane.is_moving;
                }
            }
            WindowMessage::TitleChanged(title) => {
                let window = self.windows.get_mut(&id).unwrap();
                window.title = title;
            }
            WindowMessage::PopOut(pane) => {
                let window = self.windows.get_mut(&id).unwrap();
                if let Some((popped, sibling)) = window.panes.close(&pane) {
                    window.focus = Some(sibling);

                    let (panes, _) = pane_grid::State::new(popped);
                    let window = Window {
                        panes,
                        focus: None,
                        title: format!("New window ({})", self.windows.len()),
                    };

                    let window_id = window::Id::new(self.windows.len());
                    self.windows.insert(window_id, window);
                    return window::spawn(window_id, Default::default());
                }
            }
            WindowMessage::Dragged(pane_grid::DragEvent::Dropped {
                pane,
                target,
            }) => {
                let window = self.windows.get_mut(&id).unwrap();
                window.panes.swap(&pane, &target);
            }
            // WindowMessage::Dragged(pane_grid::DragEvent::Picked { pane }) => {
            //     println!("Picked {pane:?}");
            // }
            WindowMessage::Dragged(_) => {}
            WindowMessage::TogglePin(pane) => {
                let window = self.windows.get_mut(&id).unwrap();
                if let Some(Pane { is_pinned, .. }) =
                    window.panes.get_mut(&pane)
                {
                    *is_pinned = !*is_pinned;
                }
            }
            WindowMessage::Close(pane) => {
                let window = self.windows.get_mut(&id).unwrap();
                if let Some((_, sibling)) = window.panes.close(&pane) {
                    window.focus = Some(sibling);
                }
            }
            WindowMessage::CloseFocused => {
                let window = self.windows.get_mut(&id).unwrap();
                if let Some(pane) = window.focus {
                    if let Some(Pane { is_pinned, .. }) =
                        window.panes.get(&pane)
                    {
                        if !is_pinned {
                            if let Some((_, sibling)) =
                                window.panes.close(&pane)
                            {
                                window.focus = Some(sibling);
                            }
                        }
                    }
                }
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        subscription::events_with(|event, status| {
            if let event::Status::Captured = status {
                return None;
            }

            match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    modifiers,
                    key_code,
                }) if modifiers.command() => {
                    handle_hotkey(key_code).map(|message| {
                        Message::Window(window::Id::new(0usize), message)
                    })
                } // TODO(derezzedex)
                _ => None,
            }
        })
    }

    fn close_requested(&self, window: window::Id) -> Self::Message {
        Message::Window(window, WindowMessage::CloseWindow)
    }

    fn view(&self, window_id: window::Id) -> Element<Message> {
        if let Some(window) = self.windows.get(&window_id) {
            let focus = window.focus;
            let total_panes = window.panes.len();

            let window_controls = row![
                text_input(
                    "Window title",
                    &window.title,
                    WindowMessage::TitleChanged,
                ),
                button(text("Apply")).style(theme::Button::Primary),
                button(text("Close"))
                    .on_press(WindowMessage::CloseWindow)
                    .style(theme::Button::Destructive),
            ]
            .spacing(5)
            .align_items(Alignment::Center);

            let pane_grid = PaneGrid::new(&window.panes, |id, pane| {
                let is_focused = focus == Some(id);

                let pin_button = button(
                    text(if pane.is_pinned { "Unpin" } else { "Pin" }).size(14),
                )
                .on_press(WindowMessage::TogglePin(id))
                .padding(3);

                let title = row![
                    pin_button,
                    "Pane",
                    text(pane.id.to_string()).style(if is_focused {
                        PANE_ID_COLOR_FOCUSED
                    } else {
                        PANE_ID_COLOR_UNFOCUSED
                    }),
                ]
                .spacing(5);

                let title_bar = pane_grid::TitleBar::new(title)
                    .controls(view_controls(
                        id,
                        total_panes,
                        pane.is_pinned,
                        pane.is_moving,
                        &window.title,
                        window_id,
                        &self.windows,
                    ))
                    .padding(10)
                    .style(if is_focused {
                        style::title_bar_focused
                    } else {
                        style::title_bar_active
                    });

                pane_grid::Content::new(responsive(move |size| {
                    view_content(id, total_panes, pane.is_pinned, size)
                }))
                .title_bar(title_bar)
                .style(if is_focused {
                    style::pane_focused
                } else {
                    style::pane_active
                })
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(10)
            .on_click(WindowMessage::Clicked)
            .on_drag(WindowMessage::Dragged)
            .on_resize(10, WindowMessage::Resized);

            let content: Element<_> = column![window_controls, pane_grid]
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(10)
                .into();

            return content
                .map(move |message| Message::Window(window_id, message));
        }

        container(text("This shouldn't be possible!").size(20))
            .center_x()
            .center_y()
            .into()
    }
}

const PANE_ID_COLOR_UNFOCUSED: Color = Color::from_rgb(
    0xFF as f32 / 255.0,
    0xC7 as f32 / 255.0,
    0xC7 as f32 / 255.0,
);
const PANE_ID_COLOR_FOCUSED: Color = Color::from_rgb(
    0xFF as f32 / 255.0,
    0x47 as f32 / 255.0,
    0x47 as f32 / 255.0,
);

fn handle_hotkey(key_code: keyboard::KeyCode) -> Option<WindowMessage> {
    use keyboard::KeyCode;
    use pane_grid::{Axis, Direction};

    let direction = match key_code {
        KeyCode::Up => Some(Direction::Up),
        KeyCode::Down => Some(Direction::Down),
        KeyCode::Left => Some(Direction::Left),
        KeyCode::Right => Some(Direction::Right),
        _ => None,
    };

    match key_code {
        KeyCode::V => Some(WindowMessage::SplitFocused(Axis::Vertical)),
        KeyCode::H => Some(WindowMessage::SplitFocused(Axis::Horizontal)),
        KeyCode::W => Some(WindowMessage::CloseFocused),
        _ => direction.map(WindowMessage::FocusAdjacent),
    }
}

#[derive(Debug, Clone)]
struct SelectableWindow(window::Id, String);

impl PartialEq for SelectableWindow {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for SelectableWindow {}

impl std::fmt::Display for SelectableWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.1.fmt(f)
    }
}

struct Pane {
    id: usize,
    pub axis: pane_grid::Axis,
    pub is_pinned: bool,
    pub is_moving: bool,
}

impl Pane {
    fn new(id: usize, axis: pane_grid::Axis) -> Self {
        Self {
            id,
            axis,
            is_pinned: false,
            is_moving: false,
        }
    }
}

fn view_content<'a>(
    pane: pane_grid::Pane,
    total_panes: usize,
    is_pinned: bool,
    size: Size,
) -> Element<'a, WindowMessage> {
    let button = |label, message| {
        button(
            text(label)
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Center)
                .size(16),
        )
        .width(Length::Fill)
        .padding(8)
        .on_press(message)
    };

    let mut controls = column![
        button(
            "Split horizontally",
            WindowMessage::Split(pane_grid::Axis::Horizontal, pane),
        ),
        button(
            "Split vertically",
            WindowMessage::Split(pane_grid::Axis::Vertical, pane),
        )
    ]
    .spacing(5)
    .max_width(150);

    if total_panes > 1 && !is_pinned {
        controls = controls.push(
            button("Close", WindowMessage::Close(pane))
                .style(theme::Button::Destructive),
        );
    }

    let content = column![
        text(format!("{}x{}", size.width, size.height)).size(24),
        controls,
    ]
    .width(Length::Fill)
    .spacing(10)
    .align_items(Alignment::Center);

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(5)
        .center_y()
        .into()
}

fn view_controls<'a>(
    pane: pane_grid::Pane,
    total_panes: usize,
    is_pinned: bool,
    is_moving: bool,
    window_title: &'a str,
    window_id: window::Id,
    windows: &HashMap<window::Id, Window>,
) -> Element<'a, WindowMessage> {
    let window_selector = {
        let options: Vec<_> = windows
            .iter()
            .map(|(id, window)| SelectableWindow(*id, window.title.clone()))
            .collect();
        pick_list(
            options,
            Some(SelectableWindow(window_id, window_title.to_string())),
            move |window| WindowMessage::SelectedWindow(pane, window),
        )
    };

    let mut move_to = button(text("Move to").size(14)).padding(3);

    let mut pop_out = button(text("Pop Out").size(14)).padding(3);

    let mut close = button(text("Close").size(14))
        .style(theme::Button::Destructive)
        .padding(3);

    if total_panes > 1 && !is_pinned {
        close = close.on_press(WindowMessage::Close(pane));
        pop_out = pop_out.on_press(WindowMessage::PopOut(pane));
    }

    if windows.len() > 1 && total_panes > 1 && !is_pinned {
        move_to = move_to.on_press(WindowMessage::ToggleMoving(pane));
    }

    let mut content = row![].spacing(10);
    if is_moving {
        content = content.push(pop_out).push(window_selector).push(close);
    } else {
        content = content.push(pop_out).push(move_to).push(close);
    }

    content.into()
}

mod style {
    use iced::widget::container;
    use iced::Theme;

    pub fn title_bar_active(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            text_color: Some(palette.background.strong.text),
            background: Some(palette.background.strong.color.into()),
            ..Default::default()
        }
    }

    pub fn title_bar_focused(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            text_color: Some(palette.primary.strong.text),
            background: Some(palette.primary.strong.color.into()),
            ..Default::default()
        }
    }

    pub fn pane_active(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            background: Some(palette.background.weak.color.into()),
            border_width: 2.0,
            border_color: palette.background.strong.color,
            ..Default::default()
        }
    }

    pub fn pane_focused(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            background: Some(palette.background.weak.color.into()),
            border_width: 2.0,
            border_color: palette.primary.strong.color,
            ..Default::default()
        }
    }
}
