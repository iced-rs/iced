#[derive(Debug, Clone, Copy)]
pub enum Font {
    Default,
    External {
        name: &'static str,
        bytes: &'static [u8],
    },
}
