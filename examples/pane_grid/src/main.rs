use iced::alignment::{self, Alignment};
use iced::event::{self, Event};
use iced::executor;
use iced::keyboard;
use iced::subscription;
use iced::theme::{self, Theme};
use iced::widget::pane_grid::{self, PaneGrid};
use iced::widget::{
    button, column, container, responsive, row, scrollable, text,
};
use iced::{
    Application, Color, Command, Element, Length, Settings, Size, Subscription,
};

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
    Maximize(pane_grid::Pane),
    Restore,
    Close(pane_grid::Pane),
    CloseFocused,
}

impl Application for Example {
    type Message = Message;
    type Theme = Theme;
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
            }) => match target {
                pane_grid::Target::PaneGrid(edge) => {
                    self.panes.move_to_edge(&pane, edge)
                }
                pane_grid::Target::Pane {
                    pane: target,
                    region,
                } => self.panes.split_with(&target, &pane, region),
            },
            Message::Dragged(_) => {}
            Message::TogglePin(pane) => {
                if let Some(Pane { is_pinned, .. }) = self.panes.get_mut(&pane)
                {
                    *is_pinned = !*is_pinned;
                }
            }
            Message::Maximize(pane) => self.panes.maximize(&pane),
            Message::Restore => {
                self.panes.restore();
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

    fn view(&self) -> Element<Message> {
        let focus = self.focus;
        let total_panes = self.panes.len();

        let pane_grid = PaneGrid::new(&self.panes, |id, pane, is_maximized| {
            let is_focused = focus == Some(id);

            let pin_button = button(
                text(if pane.is_pinned { "Unpin" } else { "Pin" }).size(14),
            )
            .on_press(Message::TogglePin(id))
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
                    is_maximized,
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
        .on_click(Message::Clicked)
        .on_drag(Message::Dragged)
        .on_resize(10, Message::Resized);

        container(pane_grid)
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

#[derive(Clone, Copy)]
struct Pane {
    id: usize,
    pub is_pinned: bool,
}

impl Pane {
    fn new(id: usize) -> Self {
        Self {
            id,
            is_pinned: false,
        }
    }
}

fn view_content<'a>(
    pane: pane_grid::Pane,
    total_panes: usize,
    is_pinned: bool,
    size: Size,
) -> Element<'a, Message> {
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
            Message::Split(pane_grid::Axis::Horizontal, pane),
        ),
        button(
            "Split vertically",
            Message::Split(pane_grid::Axis::Vertical, pane),
        )
    ]
    .spacing(5)
    .max_width(160);

    if total_panes > 1 && !is_pinned {
        controls = controls.push(
            button("Close", Message::Close(pane))
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
    is_maximized: bool,
) -> Element<'a, Message> {
    let mut row = row![].spacing(5);

    if total_panes > 1 {
        let toggle = {
            let (content, message) = if is_maximized {
                ("Restore", Message::Restore)
            } else {
                ("Maximize", Message::Maximize(pane))
            };
            button(text(content).size(14))
                .style(theme::Button::Secondary)
                .padding(3)
                .on_press(message)
        };

        row = row.push(toggle);
    }

    let mut close = button(text("Close").size(14))
        .style(theme::Button::Destructive)
        .padding(3);

    if total_panes > 1 && !is_pinned {
        close = close.on_press(Message::Close(pane));
    }

    row.push(close).into()
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
