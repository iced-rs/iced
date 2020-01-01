#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Settings {
    pub default_font: Option<&'static [u8]>,
}
