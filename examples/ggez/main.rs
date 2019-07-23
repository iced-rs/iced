mod renderer;
mod widget;

use renderer::Renderer;
use widget::{button, Button, Checkbox, Column, Text};

use ggez;
use ggez::event;
use ggez::graphics;
use ggez::input::mouse;

pub fn main() -> ggez::GameResult {
    let cb = ggez::ContextBuilder::new("iced", "ggez");
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut Game::new(ctx)?;
    event::run(ctx, event_loop, state)
}

struct Game {
    spritesheet: graphics::Image,

    runtime: iced::Runtime,
    button: button::State,
}

impl Game {
    fn new(context: &mut ggez::Context) -> ggez::GameResult<Game> {
        Ok(Game {
            spritesheet: graphics::Image::new(context, "/ui.png").unwrap(),

            runtime: iced::Runtime::new(),
            button: button::State::new(),
        })
    }
}

impl event::EventHandler for Game {
    fn update(&mut self, _ctx: &mut ggez::Context) -> ggez::GameResult {
        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        _context: &mut ggez::Context,
        x: f32,
        y: f32,
        _dx: f32,
        _dy: f32,
    ) {
        self.runtime.on_event(iced::Event::Mouse(
            iced::input::mouse::Event::CursorMoved { x, y },
        ));
    }

    fn draw(&mut self, context: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(context, [0.1, 0.2, 0.3, 1.0].into());

        let screen = graphics::screen_coordinates(context);

        let cursor = {
            let hello = Text::new("Hello, iced!");

            let checkbox =
                Checkbox::new(true, "Check me!", Message::CheckboxToggled);

            let button = Button::new(&mut self.button, "Press me!")
                .width(200)
                .align_self(iced::Align::End);

            let widgets = Column::new()
                .max_width(600)
                .spacing(20)
                .push(hello)
                .push(checkbox)
                .push(button);

            let content = Column::new()
                .width(screen.w as u32)
                .height(screen.h as u32)
                .align_items(iced::Align::Center)
                .justify_content(iced::Justify::Center)
                .push(widgets);

            let renderer =
                &mut Renderer::new(context, self.spritesheet.clone());

            let mut ui = self.runtime.compute(content.into(), renderer);

            let messages = ui.update();
            let cursor = ui.draw(renderer);

            renderer.flush();

            cursor
        };

        mouse::set_cursor_type(context, into_cursor_type(cursor));

        graphics::present(context)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Message {
    CheckboxToggled(bool),
}

fn into_cursor_type(cursor: iced::MouseCursor) -> mouse::MouseCursor {
    match cursor {
        iced::MouseCursor::OutOfBounds => mouse::MouseCursor::Default,
        iced::MouseCursor::Idle => mouse::MouseCursor::Default,
        iced::MouseCursor::Pointer => mouse::MouseCursor::Hand,
        iced::MouseCursor::Working => mouse::MouseCursor::Progress,
        iced::MouseCursor::Grab => mouse::MouseCursor::Grab,
        iced::MouseCursor::Grabbing => mouse::MouseCursor::Grabbing,
    }
}
