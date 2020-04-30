//! This example showcases an interactive version of the Game of Life, invented
//! by John Conway. It leverages a `Canvas` together with other widgets.
mod style;

use grid::Grid;
use iced::{
    button::{self, Button},
    executor,
    slider::{self, Slider},
    time, Align, Application, Column, Command, Container, Element, Length, Row,
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
        mouse, Color, Element, Length, Point, Rectangle, Size, Vector,
    };
    use std::collections::{HashMap, HashSet};

    #[derive(Default)]
    pub struct Grid {
        life: HashSet<Cell>,
        interaction: Option<Interaction>,
        cache: canvas::Cache,
        translation: Vector,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Message {
        Populate(Cell),
    }

    enum Interaction {
        Drawing,
        Panning { translation: Vector, start: Point },
    }

    impl Grid {
        pub fn tick(&mut self) {
            use itertools::Itertools;

            let populated_neighbors: HashMap<Cell, usize> = self
                .life
                .iter()
                .flat_map(Cell::cluster)
                .unique()
                .map(|cell| (cell, self.count_adjacent_life(cell)))
                .collect();

            for (cell, amount) in populated_neighbors.iter() {
                match amount {
                    2 => {}
                    3 => {
                        let _ = self.life.insert(*cell);
                    }
                    _ => {
                        let _ = self.life.remove(cell);
                    }
                }
            }

            self.cache.clear()
        }

        pub fn update(&mut self, message: Message) {
            match message {
                Message::Populate(cell) => {
                    self.life.insert(cell);
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

        fn count_adjacent_life(&self, cell: Cell) -> usize {
            let cluster = Cell::cluster(&cell);

            let is_neighbor = |candidate| candidate != cell;
            let is_populated = |cell| self.life.contains(&cell);

            cluster
                .filter(|&cell| is_neighbor(cell) && is_populated(cell))
                .count()
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
            let cell = Cell::at(cursor_position - self.translation);

            let populate = if self.life.contains(&cell) {
                None
            } else {
                Some(Message::Populate(cell))
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

                frame.with_save(|frame| {
                    frame.translate(self.translation);
                    frame.scale(Cell::SIZE as f32);

                    let cells = Path::new(|p| {
                        let region = Rectangle {
                            x: -self.translation.x,
                            y: -self.translation.y,
                            width: frame.width(),
                            height: frame.height(),
                        };

                        for cell in Cell::all_visible_in(region) {
                            if self.life.contains(&cell) {
                                p.rectangle(
                                    Point::new(cell.j as f32, cell.i as f32),
                                    cell_size,
                                );
                            }
                        }
                    });
                    frame.fill(&cells, Color::WHITE);
                });
            });

            let hovered_cell = {
                let mut frame = Frame::new(bounds.size());

                frame.translate(self.translation);
                frame.scale(Cell::SIZE as f32);

                if let Some(cursor_position) = cursor.position_in(&bounds) {
                    let cell = Cell::at(cursor_position - self.translation);

                    let interaction = Path::rectangle(
                        Point::new(cell.j as f32, cell.i as f32),
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

                frame.into_geometry()
            };

            vec![life, hovered_cell]
        }

        fn mouse_interaction(
            &self,
            bounds: Rectangle,
            cursor: Cursor,
        ) -> mouse::Interaction {
            match self.interaction {
                Some(Interaction::Drawing) => mouse::Interaction::Crosshair,
                Some(Interaction::Panning { .. }) => {
                    mouse::Interaction::Grabbing
                }
                None if cursor.is_over(&bounds) => {
                    mouse::Interaction::Crosshair
                }
                _ => mouse::Interaction::default(),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Cell {
        i: isize,
        j: isize,
    }

    impl Cell {
        const SIZE: usize = 20;

        fn at(position: Point) -> Cell {
            let i = (position.y / Cell::SIZE as f32).ceil() as isize;
            let j = (position.x / Cell::SIZE as f32).ceil() as isize;

            Cell {
                i: i.saturating_sub(1),
                j: j.saturating_sub(1),
            }
        }

        fn cluster(cell: &Cell) -> impl Iterator<Item = Cell> {
            use itertools::Itertools;

            let rows = cell.i.saturating_sub(1)..=cell.i.saturating_add(1);
            let columns = cell.j.saturating_sub(1)..=cell.j.saturating_add(1);

            rows.cartesian_product(columns).map(|(i, j)| Cell { i, j })
        }

        fn all_visible_in(region: Rectangle) -> impl Iterator<Item = Cell> {
            use itertools::Itertools;

            let first_row = (region.y / Cell::SIZE as f32).floor() as isize;
            let first_column = (region.x / Cell::SIZE as f32).floor() as isize;

            let visible_rows =
                (region.height / Cell::SIZE as f32).ceil() as isize;
            let visible_columns =
                (region.width / Cell::SIZE as f32).ceil() as isize;

            let rows = first_row..=first_row + visible_rows;
            let columns = first_column..=first_column + visible_columns;

            rows.cartesian_product(columns).map(|(i, j)| Cell { i, j })
        }
    }
}
