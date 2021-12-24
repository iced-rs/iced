use std::fmt;

/// An operation to be performed on some window.
pub enum Action<T> {
    /// Resize the window.
    Resize {
        /// The new logical width of the window
        width: u32,
        /// The new logical height of the window
        height: u32,
    },
    /// Move the window.
    Move {
        /// The new logical x location of the window
        x: i32,
        /// The new logical y location of the window
        y: i32,
    },
    ///Take screenshot of a headless buffer
    TakeScreenshot(Box<dyn Fn(Option<Vec<u8>>) -> T>),
}

impl<T> Action<T> {
    /// Maps the output of a clipboard [`Action`] using the provided closure.
    pub fn map<A>(self, f: impl Fn(T) -> A + 'static + Send + Sync) -> Action<A>
    where
        T: 'static,
    {
        match self {
            Self::TakeScreenshot(o) => {
                Action::TakeScreenshot(Box::new(move |s| f(o(s))))
            }
            Self::Move { x, y } => Action::Move { x, y },
            Self::Resize { width, height } => Action::Resize { width, height },
        }
    }
}

impl<T> fmt::Debug for Action<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Resize { width, height } => {
                write!(
                    f,
                    "Action::Resize. width: {}, height: {}",
                    width, height
                )
            }
            Self::Move { x, y } => {
                write!(f, "Action::Move: x: {}, y: {}", x, y,)
            }
            Self::TakeScreenshot(_) => write!(f, "Action::TakeScreenshot"),
        }
    }
}
