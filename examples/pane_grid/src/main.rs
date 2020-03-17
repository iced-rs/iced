use iced::{
    button, pane_grid, scrollable, Align, Button, Column, Container, Element,
    HorizontalAlignment, Length, PaneGrid, Sandbox, Scrollable, Settings, Text,
};
use iced_native::input::keyboard;

pub fn main() {
    Example::run(Settings::default())
}

struct Example {
    panes: pane_grid::State<Content>,
    panes_created: usize,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Split(pane_grid::Axis, pane_grid::Pane),
    SplitFocused(pane_grid::Axis),
    FocusAdjacent(pane_grid::Direction),
    Dragged(pane_grid::DragEvent),
    Resized(pane_grid::ResizeEvent),
    Close(pane_grid::Pane),
    CloseFocused,
}

impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        let (panes, _) = pane_grid::State::new(Content::new(0));

        Example {
            panes,
            panes_created: 1,
        }
    }

    fn title(&self) -> String {
        String::from("Pane grid - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Split(axis, pane) => {
                let _ = self.panes.split(
                    axis,
                    &pane,
                    Content::new(self.panes_created),
                );

                self.panes_created += 1;
            }
            Message::SplitFocused(axis) => {
                if let Some(pane) = self.panes.active() {
                    let _ = self.panes.split(
                        axis,
                        &pane,
                        Content::new(self.panes_created),
                    );

                    self.panes_created += 1;
                }
            }
            Message::FocusAdjacent(direction) => {
                if let Some(pane) = self.panes.active() {
                    if let Some(adjacent) =
                        self.panes.adjacent(&pane, direction)
                    {
                        self.panes.focus(&adjacent);
                    }
                }
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
            Message::Close(pane) => {
                let _ = self.panes.close(&pane);
            }
            Message::CloseFocused => {
                if let Some(pane) = self.panes.active() {
                    let _ = self.panes.close(&pane);
                }
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let total_panes = self.panes.len();

        let pane_grid =
            PaneGrid::new(&mut self.panes, |pane, content, focus| {
                content.view(pane, focus, total_panes)
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(5)
            .on_drag(Message::Dragged)
            .on_resize(Message::Resized)
            .on_key_press(handle_hotkey);

        Column::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .push(pane_grid)
            .into()
    }
}

fn handle_hotkey(event: pane_grid::KeyPressEvent) -> Option<Message> {
    use keyboard::KeyCode;
    use pane_grid::{Axis, Direction};

    let direction = match event.key_code {
        KeyCode::Up => Some(Direction::Up),
        KeyCode::Down => Some(Direction::Down),
        KeyCode::Left => Some(Direction::Left),
        KeyCode::Right => Some(Direction::Right),
        _ => None,
    };

    match event.key_code {
        KeyCode::V => Some(Message::SplitFocused(Axis::Vertical)),
        KeyCode::H => Some(Message::SplitFocused(Axis::Horizontal)),
        KeyCode::W => Some(Message::CloseFocused),
        _ => direction.map(Message::FocusAdjacent),
    }
}

struct Content {
    id: usize,
    scroll: scrollable::State,
    split_horizontally: button::State,
    split_vertically: button::State,
    close: button::State,
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
        focus: Option<pane_grid::Focus>,
        total_panes: usize,
    ) -> Element<Message> {
        let Content {
            id,
            scroll,
            split_horizontally,
            split_vertically,
            close,
        } = self;

        let button = |state, label, message| {
            Button::new(
                state,
                Text::new(label)
                    .width(Length::Fill)
                    .horizontal_alignment(HorizontalAlignment::Center)
                    .size(16),
            )
            .width(Length::Fill)
            .on_press(message)
        };

        let mut controls = Column::new()
            .spacing(5)
            .max_width(150)
            .push(button(
                split_horizontally,
                "Split horizontally",
                Message::Split(pane_grid::Axis::Horizontal, pane),
            ))
            .push(button(
                split_vertically,
                "Split vertically",
                Message::Split(pane_grid::Axis::Vertical, pane),
            ));

        if total_panes > 1 {
            controls =
                controls.push(button(close, "Close", Message::Close(pane)));
        }

        let content = Scrollable::new(scroll)
            .width(Length::Fill)
            .spacing(10)
            .align_items(Align::Center)
            .push(Text::new(format!("Pane {}", id)).size(30))
            .push(controls);

        Container::new(Column::new().padding(10).push(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_y()
            .style(style::Pane {
                is_focused: focus.is_some(),
            })
            .into()
    }
}

mod style {
    use iced::{container, Background, Color};

    pub struct Pane {
        pub is_focused: bool,
    }

    impl container::StyleSheet for Pane {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color(Color::WHITE)),
                border_width: if self.is_focused { 2 } else { 1 },
                border_color: if self.is_focused {
                    Color::from_rgb8(0x25, 0x7A, 0xFD)
                } else {
                    Color::BLACK
                },
                ..Default::default()
            }
        }
    }
}
