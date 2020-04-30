//! This example showcases an interactive version of the Game of Life, invented
//! by John Conway. It leverages a `Canvas` together with other widgets.
mod style;
mod time;

use grid::Grid;
use iced::{
    button::{self, Button},
    executor,
    slider::{self, Slider},
    Align, Application, Column, Command, Container, Element, Length, Row,
    Settings, Subscription, Text,
};
use std::time::{Duration, Instant};

pub fn main() {
    GameOfLife::run(Settings::default())
}

#[derive(Default)]
struct GameOfLife {
    grid: Grid,
    is_playing: bool,
    speed: u64,
    next_speed: Option<u64>,
    toggle_button: button::State,
    next_button: button::State,
    clear_button: button::State,
    speed_slider: slider::State,
}

#[derive(Debug, Clone)]
enum Message {
    Grid(grid::Message),
    Tick(Instant),
    Toggle,
    Next,
    Clear,
    SpeedChanged(f32),
}

impl Application for GameOfLife {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                speed: 1,
                ..Self::default()
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Game of Life - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Grid(message) => {
                self.grid.update(message);
            }
            Message::Tick(_) | Message::Next => {
                self.grid.tick();

                if let Some(speed) = self.next_speed.take() {
                    self.speed = speed;
                }
            }
            Message::Toggle => {
                self.is_playing = !self.is_playing;
            }
            Message::Clear => {
                self.grid = Grid::default();
            }
            Message::SpeedChanged(speed) => {
                if self.is_playing {
                    self.next_speed = Some(speed.round() as u64);
                } else {
                    self.speed = speed.round() as u64;
                }
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.is_playing {
            time::every(Duration::from_millis(1000 / self.speed))
                .map(Message::Tick)
        } else {
            Subscription::none()
        }
    }

    fn view(&mut self) -> Element<Message> {
        let playback_controls = Row::new()
            .spacing(10)
            .push(
                Button::new(
                    &mut self.toggle_button,
                    Text::new(if self.is_playing { "Pause" } else { "Play" }),
                )
                .on_press(Message::Toggle)
                .style(style::Button),
            )
            .push(
                Button::new(&mut self.next_button, Text::new("Next"))
                    .on_press(Message::Next)
                    .style(style::Button),
            )
            .push(
                Button::new(&mut self.clear_button, Text::new("Clear"))
                    .on_press(Message::Clear)
                    .style(style::Button),
            );

        let selected_speed = self.next_speed.unwrap_or(self.speed);
        let speed_controls = Row::new()
            .spacing(10)
            .push(
                Slider::new(
                    &mut self.speed_slider,
                    1.0..=20.0,
                    selected_speed as f32,
                    Message::SpeedChanged,
                )
                .width(Length::Units(200))
                .style(style::Slider),
            )
            .push(Text::new(format!("x{}", selected_speed)).size(16))
            .align_items(Align::Center);

        let controls = Row::new()
            .padding(10)
            .spacing(20)
            .push(playback_controls)
            .push(speed_controls);

        let content = Column::new()
            .spacing(10)
            .align_items(Align::Center)
            .push(self.grid.view().map(Message::Grid))
            .push(controls);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style::Container)
            .into()
    }
}

mod grid {
    use iced::{
        canvas::{self, Canvas, Cursor, Event, Frame, Geometry, Path},
        mouse, Color, Element, Length, MouseCursor, Point, Rectangle, Size,
        Vector,
    };
    use std::collections::{HashMap, HashSet};

    const CELL_SIZE: usize = 20;

    #[derive(Default)]
    pub struct Grid {
        alive_cells: HashSet<(isize, isize)>,
        interaction: Option<Interaction>,
        cache: canvas::Cache,
        translation: Vector,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Message {
        Populate { cell: (isize, isize) },
    }

    enum Interaction {
        Drawing,
        Panning { translation: Vector, start: Point },
    }

    impl Grid {
        fn with_neighbors(
            i: isize,
            j: isize,
        ) -> impl Iterator<Item = (isize, isize)> {
            use itertools::Itertools;

            let rows = i.saturating_sub(1)..=i.saturating_add(1);
            let columns = j.saturating_sub(1)..=j.saturating_add(1);

            rows.cartesian_product(columns)
        }

        pub fn tick(&mut self) {
            use itertools::Itertools;

            let populated_neighbors: HashMap<(isize, isize), usize> = self
                .alive_cells
                .iter()
                .flat_map(|&(i, j)| Self::with_neighbors(i, j))
                .unique()
                .map(|(i, j)| ((i, j), self.populated_neighbors(i, j)))
                .collect();

            for (&(i, j), amount) in populated_neighbors.iter() {
                let is_populated = self.alive_cells.contains(&(i, j));

                match amount {
                    2 | 3 if is_populated => {}
                    3 => {
                        let _ = self.alive_cells.insert((i, j));
                    }
                    _ if is_populated => {
                        let _ = self.alive_cells.remove(&(i, j));
                    }
                    _ => {}
                }
            }

            self.cache.clear()
        }

        pub fn update(&mut self, message: Message) {
            match message {
                Message::Populate { cell } => {
                    self.alive_cells.insert(cell);
                    self.cache.clear()
                }
            }
        }

        pub fn view<'a>(&'a mut self) -> Element<'a, Message> {
            Canvas::new(self)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }

        fn populated_neighbors(&self, row: isize, column: isize) -> usize {
            let with_neighbors = Self::with_neighbors(row, column);

            let is_neighbor = |i: isize, j: isize| i != row || j != column;
            let is_populated =
                |i: isize, j: isize| self.alive_cells.contains(&(i, j));

            with_neighbors
                .filter(|&(i, j)| is_neighbor(i, j) && is_populated(i, j))
                .count()
        }

        fn cell_at(&self, position: Point) -> Option<(isize, isize)> {
            let i = (position.y / CELL_SIZE as f32).ceil() as isize;
            let j = (position.x / CELL_SIZE as f32).ceil() as isize;

            Some((i.saturating_sub(1), j.saturating_sub(1)))
        }
    }

    impl<'a> canvas::Program<Message> for Grid {
        fn update(
            &mut self,
            event: Event,
            bounds: Rectangle,
            cursor: Cursor,
        ) -> Option<Message> {
            if let Event::Mouse(mouse::Event::ButtonReleased(_)) = event {
                self.interaction = None;
            }

            let cursor_position = cursor.position_in(&bounds)?;
            let cell = self.cell_at(cursor_position - self.translation)?;

            let populate = if self.alive_cells.contains(&cell) {
                None
            } else {
                Some(Message::Populate { cell })
            };

            match event {
                Event::Mouse(mouse_event) => match mouse_event {
                    mouse::Event::ButtonPressed(button) => match button {
                        mouse::Button::Left => {
                            self.interaction = Some(Interaction::Drawing);

                            populate
                        }
                        mouse::Button::Right => {
                            self.interaction = Some(Interaction::Panning {
                                translation: self.translation,
                                start: cursor_position,
                            });

                            None
                        }
                        _ => None,
                    },
                    mouse::Event::CursorMoved { .. } => {
                        match self.interaction {
                            Some(Interaction::Drawing) => populate,
                            Some(Interaction::Panning {
                                translation,
                                start,
                            }) => {
                                self.translation =
                                    translation + (cursor_position - start);

                                self.cache.clear();

                                None
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                },
            }
        }

        fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry> {
            let cell_size = Size::new(1.0, 1.0);

            let life = self.cache.draw(bounds.size(), |frame| {
                let background = Path::rectangle(Point::ORIGIN, frame.size());
                frame.fill(
                    &background,
                    Color::from_rgb(
                        0x40 as f32 / 255.0,
                        0x44 as f32 / 255.0,
                        0x4B as f32 / 255.0,
                    ),
                );

                let first_row =
                    (-self.translation.y / CELL_SIZE as f32).floor() as isize;
                let first_column =
                    (-self.translation.x / CELL_SIZE as f32).floor() as isize;

                let visible_rows =
                    (frame.height() / CELL_SIZE as f32).ceil() as isize;
                let visible_columns =
                    (frame.width() / CELL_SIZE as f32).ceil() as isize;

                frame.with_save(|frame| {
                    frame.translate(self.translation);
                    frame.scale(CELL_SIZE as f32);

                    let cells = Path::new(|p| {
                        for i in first_row..=(first_row + visible_rows) {
                            for j in
                                first_column..=(first_column + visible_columns)
                            {
                                if self.alive_cells.contains(&(i, j)) {
                                    p.rectangle(
                                        Point::new(j as f32, i as f32),
                                        cell_size,
                                    );
                                }
                            }
                        }
                    });
                    frame.fill(&cells, Color::WHITE);
                });
            });

            let hovered_cell = {
                let mut frame = Frame::new(bounds.size());

                frame.translate(self.translation);
                frame.scale(CELL_SIZE as f32);

                if let Some(cursor_position) = cursor.position_in(&bounds) {
                    if let Some((i, j)) =
                        self.cell_at(cursor_position - self.translation)
                    {
                        let interaction = Path::rectangle(
                            Point::new(j as f32, i as f32),
                            cell_size,
                        );

                        frame.fill(
                            &interaction,
                            Color {
                                a: 0.5,
                                ..Color::BLACK
                            },
                        );
                    }
                }

                frame.into_geometry()
            };

            vec![life, hovered_cell]
        }

        fn mouse_cursor(
            &self,
            bounds: Rectangle,
            cursor: Cursor,
        ) -> MouseCursor {
            match self.interaction {
                Some(Interaction::Drawing) => MouseCursor::Crosshair,
                Some(Interaction::Panning { .. }) => MouseCursor::Grabbing,
                None if cursor.is_over(&bounds) => MouseCursor::Crosshair,
                _ => MouseCursor::default(),
            }
        }
    }
}
