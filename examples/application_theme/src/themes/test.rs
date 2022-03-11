use crate::button::Style;
use iced::button;
use iced::{Color, Theme};

struct MyColorPalette {
    text: Color,
    buttons: Color,
}

struct MyTheme;

impl Theme for MyTheme {
    type ColorPalette = MyColorPalette;
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
