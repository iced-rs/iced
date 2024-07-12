use iced::widget::{center, column, row, slider, text};
use iced::{Center, Element};

use std::time::Duration;

mod circular;
mod easing;
mod linear;

use circular::Circular;
use linear::Linear;

pub fn main() -> iced::Result {
    iced::application(
        "Loading Spinners - Iced",
        LoadingSpinners::update,
        LoadingSpinners::view,
    )
    .antialiasing(true)
    .run()
}

struct LoadingSpinners {
    cycle_duration: f32,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    CycleDurationChanged(f32),
}

impl LoadingSpinners {
    fn update(&mut self, message: Message) {
        match message {
            Message::CycleDurationChanged(duration) => {
                self.cycle_duration = duration;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let column = [
            &easing::EMPHASIZED,
            &easing::EMPHASIZED_DECELERATE,
            &easing::EMPHASIZED_ACCELERATE,
            &easing::STANDARD,
            &easing::STANDARD_DECELERATE,
            &easing::STANDARD_ACCELERATE,
        ]
        .iter()
        .zip([
            "Emphasized:",
            "Emphasized Decelerate:",
            "Emphasized Accelerate:",
            "Standard:",
            "Standard Decelerate:",
            "Standard Accelerate:",
        ])
        .fold(column![], |column, (easing, label)| {
            column.push(
                row![
                    text(label).width(250),
                    Linear::new().easing(easing).cycle_duration(
                        Duration::from_secs_f32(self.cycle_duration)
                    ),
                    Circular::new().easing(easing).cycle_duration(
                        Duration::from_secs_f32(self.cycle_duration)
                    )
                ]
                .align_y(Center)
                .spacing(20.0),
            )
        })
        .spacing(20);

        center(
            column.push(
                row![
                    text("Cycle duration:"),
                    slider(1.0..=1000.0, self.cycle_duration * 100.0, |x| {
                        Message::CycleDurationChanged(x / 100.0)
                    })
                    .width(200.0),
                    text!("{:.2}s", self.cycle_duration),
                ]
                .align_y(Center)
                .spacing(20.0),
            ),
        )
        .into()
    }
}

impl Default for LoadingSpinners {
    fn default() -> Self {
        Self {
            cycle_duration: 2.0,
        }
    }
}
