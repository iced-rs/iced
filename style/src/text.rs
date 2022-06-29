use iced_core::Color;

pub trait StyleSheet {
    type Style: Default + Copy;

    fn appearance(&self, style: Self::Style) -> Appearance;
}

#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    pub color: Option<Color>,
}

impl Default for Appearance {
    fn default() -> Self {
        Self { color: None }
    }
}
