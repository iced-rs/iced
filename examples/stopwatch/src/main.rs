use iced::{Application, Settings};
use stopwatch::Stopwatch;

pub fn main() {
    Stopwatch::run(Settings::default())
}
