use std::{fmt::Debug, ops::Add, sync::Arc};

/// The size of a window upon creation.
#[derive(Clone)]
pub enum Size {
    /// Sets the size of the window directly.
    Fixed(crate::Size),
    /// Allows you to set the size of the window based on the monitors resolution
    ///
    /// The function receives the the monitor's resolution as input.
    FromScreensize(Arc<dyn Send + Sync + Fn(crate::Size) -> crate::Size>),
}

impl Size {
    /// Returns the default fixed windows size.
    /// The output is an [`iced::Size`].
    ///
    /// If you want to populate the [`iced::window::Settings`], you can use the `default()` function instead.
    pub fn default_window_size() -> crate::Size {
        crate::Size::new(1024.0, 768.0)
    }

    fn from_screen_size(
        func: impl 'static + Send + Sync + Fn(crate::Size) -> crate::Size,
    ) -> Self {
        Self::FromScreensize(Arc::new(func))
    }
}

impl Default for Size {
    fn default() -> Self {
        Self::Fixed(Self::default_window_size())
    }
}

impl Debug for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fixed(size) => write!(f, "Fixed({:?})", size),
            Self::FromScreensize(_) => {
                write!(f, "FromScreensize(...)")
            }
        }
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

impl Add<crate::Size> for Size {
    type Output = Self;

    fn add(self, other: crate::Size) -> Self {
        match self {
            Self::Fixed(size) => Self::Fixed(size + other),
            Self::FromScreensize(func) => {
                Self::from_screen_size(move |size| func(size) + other)
            }
        }
    }
}
