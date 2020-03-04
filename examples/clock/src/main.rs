use clock::Clock;
use iced::{Application, Settings};

pub fn main() {
    Clock::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}
