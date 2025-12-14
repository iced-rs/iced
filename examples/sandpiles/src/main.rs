use iced::mouse;
use iced::time;
use iced::widget::canvas;
use iced::{Element, Event, Fill, Point, Rectangle, Renderer, Size, Subscription, Theme, Vector};

use std::collections::{HashMap, HashSet};

pub fn main() -> iced::Result {
    iced::application(Sandpiles::new, Sandpiles::update, Sandpiles::view)
        .subscription(Sandpiles::subscription)
        .run()
}

struct Sandpiles {
    grid: Grid,
    sandfalls: HashSet<Cell>,
    cache: canvas::Cache,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Add(Cell),
    Tick,
}

impl Sandpiles {
    fn new() -> Self {
        Self {
            grid: Grid::new(),
            sandfalls: HashSet::from_iter([Cell::ORIGIN]),
            cache: canvas::Cache::new(),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Add(sandfall) => {
                self.sandfalls.insert(sandfall);
            }
            Message::Tick => {
                for sandfall in &self.sandfalls {
                    self.grid.add(*sandfall, 1);
                }

                self.grid.topple();
                self.cache.clear();
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        canvas(Viewer {
            grid: &self.grid,
            cache: &self.cache,
        })
        .width(Fill)
        .height(Fill)
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.sandfalls.is_empty() {
            return Subscription::none();
        }

        time::every(time::milliseconds(50)).map(|_| Message::Tick)
    }
}

#[derive(Debug)]
struct Grid {
    sand: HashMap<Cell, u32>,
    saturated: HashSet<Cell>,
}

impl Grid {
    pub fn new() -> Self {
        Self {
            sand: HashMap::new(),
            saturated: HashSet::new(),
        }
    }

    pub fn add(&mut self, cell: Cell, amount: u32) {
        let grains = self.sand.entry(cell).or_default();

        *grains += amount;

        if *grains >= 4 {
            self.saturated.insert(cell);
        }
    }

    pub fn topple(&mut self) {
        loop {
            let Some(cell) = self.saturated.iter().next().copied() else {
                return;
            };

            let grains = self.sand.entry(cell).or_default();
            let amount = *grains / 4;
            *grains %= 4;

            for neighbor in cell.neighbors() {
                self.add(neighbor, amount);
            }

            let _ = self.saturated.remove(&cell);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Cell {
    row: isize,
    column: isize,
}

impl Cell {
    pub const ORIGIN: Self = Self { row: 0, column: 0 };

    pub fn neighbors(self) -> impl Iterator<Item = Cell> {
        [(0, -1), (-1, 0), (1, 0), (0, 1)]
            .into_iter()
            .map(move |(i, j)| Cell {
                row: self.row + i,
                column: self.column + j,
            })
    }
}

struct Viewer<'a> {
    grid: &'a Grid,
    cache: &'a canvas::Cache,
}

impl Viewer<'_> {
    const CELL_SIZE: f32 = 10.0;
}

impl canvas::Program<Message> for Viewer<'_> {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let position = cursor.position_in(bounds)? - (bounds.center() - Point::ORIGIN);
                let row = (position.x / Self::CELL_SIZE).round() as isize;
                let column = (position.y / Self::CELL_SIZE).round() as isize;

                Some(canvas::Action::publish(Message::Add(Cell { row, column })))
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) if cursor.is_over(bounds) => {
                Some(canvas::Action::request_redraw())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let palette = theme.extended_palette();

            let cells_x = (frame.width() / Self::CELL_SIZE).ceil() as isize;
            let cells_y = (frame.height() / Self::CELL_SIZE).ceil() as isize;

            let rows = -cells_x / 2..cells_x / 2;
            let columns = -cells_y / 2..cells_y / 2;

            frame.translate(
                frame.center()
                    - Point::ORIGIN
                    - Vector::new(Self::CELL_SIZE, Self::CELL_SIZE) / 2.0,
            );

            for row in rows {
                for column in columns.clone() {
                    let grains = self
                        .grid
                        .sand
                        .get(&Cell { row, column })
                        .copied()
                        .unwrap_or_default();

                    if grains == 0 {
                        continue;
                    }

                    frame.fill_rectangle(
                        Point::new(
                            row as f32 * Self::CELL_SIZE,
                            column as f32 * Self::CELL_SIZE,
                        ),
                        Size::new(Self::CELL_SIZE, Self::CELL_SIZE),
                        match grains {
                            3.. => palette.background.strongest.color,
                            2 => palette.background.strong.color,
                            _ => palette.background.weak.color,
                        },
                    );
                }
            }
        });

        let cursor = {
            let mut frame = canvas::Frame::new(renderer, bounds.size());

            if let Some(position) = cursor.position_in(bounds) {
                let translation = frame.center() - Point::ORIGIN;
                let position = position - translation;

                frame.translate(translation - Vector::new(Self::CELL_SIZE, Self::CELL_SIZE) / 2.0);
                frame.fill_rectangle(
                    Point::new(
                        (position.x / Self::CELL_SIZE).round() * Self::CELL_SIZE,
                        (position.y / Self::CELL_SIZE).round() * Self::CELL_SIZE,
                    ),
                    Size::new(Self::CELL_SIZE, Self::CELL_SIZE),
                    theme.palette().primary,
                );
            }

            frame.into_geometry()
        };

        vec![geometry, cursor]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::None
        }
    }
}
