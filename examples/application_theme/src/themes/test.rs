use crate::button::Style;
use iced::button;
use iced::{Color, Styling};

struct MyColorPalette {
    text: Color,
    buttons: Color,
}

struct MyTheme;

impl Styling for MyTheme {
    type Theme = MyColorPalette;
}

impl button::StyleSheet<MyColorPalette> for MyTheme {
    fn active(&self, color_palette: &MyColorPalette) -> Style {
        todo!()
    }

    fn hovered(&self, color_palette: &MyColorPalette) -> Style {
        todo!()
    }

    fn pressed(&self, color_palette: &MyColorPalette) -> Style {
        todo!()
    }

    fn disabled(&self, color_palette: &MyColorPalette) -> Style {
        todo!()
    }
}
