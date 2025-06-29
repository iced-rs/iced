/// The size of a window upon creation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    /// Sets the size of the window directly.
    Fixed(crate::Size),
    /// Allows you to set the size of the window based on the monitors resolution
    ///
    /// The function receives the the monitor's resolution as input.
    FromScreensize(fn(crate::Size) -> crate::Size),
}

impl Size {
    /// Returns the default fixed windows size.
    /// The output is an [`iced::Size`].
    ///
    /// If you want to populate the [`iced::window::Settings`], you can use the `default()` function instead.
    pub fn default_window_size() -> crate::Size {
        crate::Size::new(1024.0, 768.0)
    }
}

impl Default for Size {
    fn default() -> Self {
        Self::Fixed(Self::default_window_size())
    }
}

impl<Source> From<Source> for Size
where
    Source: Into<crate::Size>,
{
    fn from(value: Source) -> Self {
        Self::Fixed(value.into())
    }
}
