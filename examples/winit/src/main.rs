use iced::{Column, Element, Sandbox, Settings, window::Settings as WindowSettings};

const WINDOW_WIDTH: i32 = 200;
const WINDOW_HEIGHT: i32 = 200;
const DISPLAY_WIDTH: i32 = 1920;
const DISPLAY_HEIGHT: i32 = 1080;
// These numbers are specific to a 1920x1080 monitor
const BORDER_X: i32 = 8;
const BORDER_Y: i32 = 2;
const CAPTION_HEIGHT: i32 = 4;

pub fn main() {
    let x = DISPLAY_WIDTH / 2 - WINDOW_WIDTH / 2 - BORDER_X;
    let y = DISPLAY_HEIGHT / 2 - WINDOW_HEIGHT / 2 - BORDER_Y - CAPTION_HEIGHT;
    let settings = Settings {
        window: WindowSettings {
            size: (WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32),
            position: (x, y),
            ..Default::default()
        },
        ..Default::default()
    };
    Winit::run(settings).unwrap()
}

#[derive(Default)]
struct Winit;

impl Sandbox for Winit {
    type Message = ();

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("winit - Iced")
    }

    fn update(&mut self, _message: Self::Message) {
    }

    fn view(&mut self) -> Element<Self::Message> {
        Column::new().into()
    }
}
