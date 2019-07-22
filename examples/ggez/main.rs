mod renderer;
mod widget;

use renderer::Renderer;
use widget::{button, Button, Column, Text};

use ggez;
use ggez::event;
use ggez::graphics;
use ggez::input::mouse;

use iced::Interface;

pub fn main() -> ggez::GameResult {
    let cb = ggez::ContextBuilder::new("iced", "ggez");
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut Game::new(ctx)?;
    event::run(ctx, event_loop, state)
}

struct Game {
    spritesheet: graphics::Image,
    button: button::State,
}

impl Game {
    fn new(context: &mut ggez::Context) -> ggez::GameResult<Game> {
        Ok(Game {
            spritesheet: graphics::Image::new(context, "/ui.png").unwrap(),
            button: button::State::new(),
        })
    }
}

impl event::EventHandler for Game {
    fn update(&mut self, _ctx: &mut ggez::Context) -> ggez::GameResult {
        Ok(())
    }

    fn draw(&mut self, context: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(context, [0.1, 0.2, 0.3, 1.0].into());

        let screen = graphics::screen_coordinates(context);

        let cursor = {
            let hello = Text::new("Hello, iced!")
                .horizontal_alignment(iced::text::HorizontalAlignment::Center);

            let button = Button::new(&mut self.button, "Press me!").width(200);

            let content = Column::new()
                .width(screen.w as u32)
                .height(screen.h as u32)
                .align_items(iced::Align::Center)
                .justify_content(iced::Justify::Center)
                .spacing(20)
                .push(hello)
                .push(button);

            let renderer =
                &mut Renderer::new(context, self.spritesheet.clone());

            let ui: Interface<(), Renderer> =
                Interface::compute(content.into(), renderer);

            let cursor = ui.draw(renderer, iced::Point::new(0.0, 0.0));

            renderer.flush();

            cursor
        };

        mouse::set_cursor_type(context, into_cursor_type(cursor));

        graphics::present(context)?;
        Ok(())
    }
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
