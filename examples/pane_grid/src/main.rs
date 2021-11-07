use iced::{
    alignment, button, executor, keyboard, pane_grid, scrollable, Alignment,
    Application, Button, Color, Column, Command, Container, Element, Length,
    PaneGrid, Row, Scrollable, Settings, Subscription, Text,
};
use iced_native::{event, subscription, Event};

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

struct Example {
    panes: pane_grid::State<Pane>,
    panes_created: usize,
    focus: Option<pane_grid::Pane>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Split(pane_grid::Axis, pane_grid::Pane),
    SplitFocused(pane_grid::Axis),
    FocusAdjacent(pane_grid::Direction),
    Clicked(pane_grid::Pane),
    Dragged(pane_grid::DragEvent),
    Resized(pane_grid::ResizeEvent),
    TogglePin(pane_grid::Pane),
    Close(pane_grid::Pane),
    CloseFocused,
}

impl Application for Example {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let (panes, _) = pane_grid::State::new(Pane::new(0));

        (
            Example {
                panes,
                panes_created: 1,
                focus: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Pane grid - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Split(axis, pane) => {
                let result = self.panes.split(
                    axis,
                    &pane,
                    Pane::new(self.panes_created),
                );

                if let Some((pane, _)) = result {
                    self.focus = Some(pane);
                }

                self.panes_created += 1;
            }
            Message::SplitFocused(axis) => {
                if let Some(pane) = self.focus {
                    let result = self.panes.split(
                        axis,
                        &pane,
                        Pane::new(self.panes_created),
                    );

                    if let Some((pane, _)) = result {
                        self.focus = Some(pane);
                    }

                    self.panes_created += 1;
                }
            }
            Message::FocusAdjacent(direction) => {
                if let Some(pane) = self.focus {
                    if let Some(adjacent) =
                        self.panes.adjacent(&pane, direction)
                    {
                        self.focus = Some(adjacent);
                    }
                }
            }
            Message::Clicked(pane) => {
                self.focus = Some(pane);
            }
            Message::Resized(pane_grid::ResizeEvent { split, ratio }) => {
                self.panes.resize(&split, ratio);
            }
            Message::Dragged(pane_grid::DragEvent::Dropped {
                pane,
                target,
            }) => {
                self.panes.swap(&pane, &target);
            }
            Message::Dragged(_) => {}
            Message::TogglePin(pane) => {
                if let Some(Pane { is_pinned, .. }) = self.panes.get_mut(&pane)
                {
                    *is_pinned = !*is_pinned;
                }
            }
            Message::Close(pane) => {
                if let Some((_, sibling)) = self.panes.close(&pane) {
                    self.focus = Some(sibling);
                }
            }
            Message::CloseFocused => {
                if let Some(pane) = self.focus {
                    if let Some(Pane { is_pinned, .. }) = self.panes.get(&pane)
                    {
                        if !is_pinned {
                            if let Some((_, sibling)) = self.panes.close(&pane)
                            {
                                self.focus = Some(sibling);
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
                }) if modifiers.command() => handle_hotkey(key_code),
                _ => None,
            }
        })
    }

    fn view(&mut self) -> Element<Message> {
        let focus = self.focus;
        let total_panes = self.panes.len();

        let pane_grid = PaneGrid::new(&mut self.panes, |id, pane| {
            let is_focused = focus == Some(id);

            let text = if pane.is_pinned { "Unpin" } else { "Pin" };
            let pin_button =
                Button::new(&mut pane.pin_button, Text::new(text).size(14))
                    .on_press(Message::TogglePin(id))
                    .style(style::Button::Pin)
                    .padding(3);

            let title = Row::with_children(vec![
                pin_button.into(),
                Text::new("Pane").into(),
                Text::new(pane.content.id.to_string())
                    .color(if is_focused {
                        PANE_ID_COLOR_FOCUSED
                    } else {
                        PANE_ID_COLOR_UNFOCUSED
                    })
                    .into(),
            ])
            .spacing(5);

            let title_bar = pane_grid::TitleBar::new(title)
                .controls(pane.controls.view(id, total_panes, pane.is_pinned))
                .padding(10)
                .style(if is_focused {
                    style::TitleBar::Focused
                } else {
                    style::TitleBar::Active
                });

            pane_grid::Content::new(pane.content.view(
                id,
                total_panes,
                pane.is_pinned,
            ))
            .title_bar(title_bar)
            .style(if is_focused {
                style::Pane::Focused
            } else {
                style::Pane::Active
            })
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(10)
        .on_click(Message::Clicked)
        .on_drag(Message::Dragged)
        .on_resize(10, Message::Resized);

        Container::new(pane_grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
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

fn handle_hotkey(key_code: keyboard::KeyCode) -> Option<Message> {
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
        KeyCode::V => Some(Message::SplitFocused(Axis::Vertical)),
        KeyCode::H => Some(Message::SplitFocused(Axis::Horizontal)),
        KeyCode::W => Some(Message::CloseFocused),
        _ => direction.map(Message::FocusAdjacent),
    }
}

struct Pane {
    pub is_pinned: bool,
    pub pin_button: button::State,
    pub content: Content,
    pub controls: Controls,
}

struct Content {
    id: usize,
    scroll: scrollable::State,
    split_horizontally: button::State,
    split_vertically: button::State,
    close: button::State,
}

struct Controls {
    close: button::State,
}

impl Pane {
    fn new(id: usize) -> Self {
        Self {
            is_pinned: false,
            pin_button: button::State::new(),
            content: Content::new(id),
            controls: Controls::new(),
        }
    }
}

impl Content {
    fn new(id: usize) -> Self {
        Content {
            id,
            scroll: scrollable::State::new(),
            split_horizontally: button::State::new(),
            split_vertically: button::State::new(),
            close: button::State::new(),
        }
    }
    fn view(
        &mut self,
        pane: pane_grid::Pane,
        total_panes: usize,
        is_pinned: bool,
    ) -> Element<Message> {
        let Content {
            scroll,
            split_horizontally,
            split_vertically,
            close,
            ..
        } = self;

        let button = |state, label, message, style| {
            Button::new(
                state,
                Text::new(label)
                    .width(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .size(16),
            )
            .width(Length::Fill)
            .padding(8)
            .on_press(message)
            .style(style)
        };

        let mut controls = Column::new()
            .spacing(5)
            .max_width(150)
            .push(button(
                split_horizontally,
                "Split horizontally",
                Message::Split(pane_grid::Axis::Horizontal, pane),
                style::Button::Primary,
            ))
            .push(button(
                split_vertically,
                "Split vertically",
                Message::Split(pane_grid::Axis::Vertical, pane),
                style::Button::Primary,
            ));

        if total_panes > 1 && !is_pinned {
            controls = controls.push(button(
                close,
                "Close",
                Message::Close(pane),
                style::Button::Destructive,
            ));
        }

        let content = Scrollable::new(scroll)
            .width(Length::Fill)
            .spacing(10)
            .align_items(Alignment::Center)
            .push(controls);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(5)
            .center_y()
            .into()
    }
}

impl Controls {
    fn new() -> Self {
        Self {
            close: button::State::new(),
        }
    }

    pub fn view(
        &mut self,
        pane: pane_grid::Pane,
        total_panes: usize,
        is_pinned: bool,
    ) -> Element<Message> {
        let mut button =
            Button::new(&mut self.close, Text::new("Close").size(14))
                .style(style::Button::Control)
                .padding(3);
        if total_panes > 1 && !is_pinned {
            button = button.on_press(Message::Close(pane));
        }
        button.into()
    }
}

mod style {
    use crate::PANE_ID_COLOR_FOCUSED;
    use iced::{button, container, Background, Color, Vector};

    const SURFACE: Color = Color::from_rgb(
        0xF2 as f32 / 255.0,
        0xF3 as f32 / 255.0,
        0xF5 as f32 / 255.0,
    );

    const ACTIVE: Color = Color::from_rgb(
        0x72 as f32 / 255.0,
        0x89 as f32 / 255.0,
        0xDA as f32 / 255.0,
    );

    const HOVERED: Color = Color::from_rgb(
        0x67 as f32 / 255.0,
        0x7B as f32 / 255.0,
        0xC4 as f32 / 255.0,
    );

    pub enum TitleBar {
        Active,
        Focused,
    }

    impl container::StyleSheet for TitleBar {
        fn style(&self) -> container::Style {
            let pane = match self {
                Self::Active => Pane::Active,
                Self::Focused => Pane::Focused,
            }
            .style();

            container::Style {
                text_color: Some(Color::WHITE),
                background: Some(pane.border_color.into()),
                ..Default::default()
            }
        }
    }

    pub enum Pane {
        Active,
        Focused,
    }

    impl container::StyleSheet for Pane {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color(SURFACE)),
                border_width: 2.0,
                border_color: match self {
                    Self::Active => Color::from_rgb(0.7, 0.7, 0.7),
                    Self::Focused => Color::BLACK,
                },
                ..Default::default()
            }
        }
    }

    pub enum Button {
        Primary,
        Destructive,
        Control,
        Pin,
    }

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            let (background, text_color) = match self {
                Button::Primary => (Some(ACTIVE), Color::WHITE),
                Button::Destructive => {
                    (None, Color::from_rgb8(0xFF, 0x47, 0x47))
                }
                Button::Control => (Some(PANE_ID_COLOR_FOCUSED), Color::WHITE),
                Button::Pin => (Some(ACTIVE), Color::WHITE),
            };

            button::Style {
                text_color,
                background: background.map(Background::Color),
                border_radius: 5.0,
                shadow_offset: Vector::new(0.0, 0.0),
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            let active = self.active();

            let background = match self {
                Button::Primary => Some(HOVERED),
                Button::Destructive => Some(Color {
                    a: 0.2,
                    ..active.text_color
                }),
                Button::Control => Some(PANE_ID_COLOR_FOCUSED),
                Button::Pin => Some(HOVERED),
            };

            button::Style {
                background: background.map(Background::Color),
                ..active
            }
        }
    }
}
