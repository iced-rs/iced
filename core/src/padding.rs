//! Space stuff around the perimeter.
use crate::{Pixels, Size};

/// An amount of space to pad for each side of a box
///
/// You can leverage the `From` trait to build [`Padding`] conveniently:
///
/// ```
/// # use iced_core::Padding;
/// #
/// let padding = Padding::from(20);              // 20px on all sides
/// let padding = Padding::from([10, 20]);        // top/bottom, left/right
/// ```
///
/// Normally, the `padding` method of a widget will ask for an `Into<Padding>`,
/// so you can easily write:
///
/// ```
/// # use iced_core::Padding;
/// #
/// # struct Widget;
/// #
/// impl Widget {
///     # pub fn new() -> Self { Self }
///     #
///     pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
///         // ...
///         self
///     }
/// }
///
/// let widget = Widget::new().padding(20);              // 20px on all sides
/// let widget = Widget::new().padding([10, 20]);        // top/bottom, left/right
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Padding {
    /// Top padding
    pub top: f32,
    /// Right padding
    pub right: f32,
    /// Bottom padding
    pub bottom: f32,
    /// Left padding
    pub left: f32,
}

/// Create a [`Padding`] that is equal on all sides.
pub fn all(padding: impl Into<Pixels>) -> Padding {
    Padding::new(padding.into().0)
}

/// Create some top [`Padding`].
pub fn top(padding: impl Into<Pixels>) -> Padding {
    Padding::default().top(padding)
}

/// Create some bottom [`Padding`].
pub fn bottom(padding: impl Into<Pixels>) -> Padding {
    Padding::default().bottom(padding)
}

/// Create some left [`Padding`].
pub fn left(padding: impl Into<Pixels>) -> Padding {
    Padding::default().left(padding)
}

/// Create some right [`Padding`].
pub fn right(padding: impl Into<Pixels>) -> Padding {
    Padding::default().right(padding)
}

impl Padding {
    /// Padding of zero
    pub const ZERO: Padding = Padding {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };

    /// Create a [`Padding`] that is equal on all sides.
    pub const fn new(padding: f32) -> Padding {
        Padding {
            top: padding,
            right: padding,
            bottom: padding,
            left: padding,
        }
    }

    /// Sets the [`top`] of the [`Padding`].
    ///
    /// [`top`]: Self::top
    pub fn top(self, top: impl Into<Pixels>) -> Self {
        Self {
            top: top.into().0,
            ..self
        }
    }

    /// Sets the [`bottom`] of the [`Padding`].
    ///
    /// [`bottom`]: Self::bottom
    pub fn bottom(self, bottom: impl Into<Pixels>) -> Self {
        Self {
            bottom: bottom.into().0,
            ..self
        }
    }

    /// Sets the [`left`] of the [`Padding`].
    ///
    /// [`left`]: Self::left
    pub fn left(self, left: impl Into<Pixels>) -> Self {
        Self {
            left: left.into().0,
            ..self
        }
    }

    /// Sets the [`right`] of the [`Padding`].
    ///
    /// [`right`]: Self::right
    pub fn right(self, right: impl Into<Pixels>) -> Self {
        Self {
            right: right.into().0,
            ..self
        }
    }

    /// Returns the total amount of vertical [`Padding`].
    pub fn vertical(self) -> f32 {
        self.top + self.bottom
    }

    /// Returns the total amount of horizontal [`Padding`].
    pub fn horizontal(self) -> f32 {
        self.left + self.right
    }

    /// Fits the [`Padding`] between the provided `inner` and `outer` [`Size`].
    pub fn fit(self, inner: Size, outer: Size) -> Self {
        let available = (outer - inner).max(Size::ZERO);
        let new_top = self.top.min(available.height);
        let new_left = self.left.min(available.width);

        Padding {
            top: new_top,
            bottom: self.bottom.min(available.height - new_top),
            left: new_left,
            right: self.right.min(available.width - new_left),
        }
    }
}

impl From<u16> for Padding {
    fn from(p: u16) -> Self {
        Padding {
            top: f32::from(p),
            right: f32::from(p),
            bottom: f32::from(p),
            left: f32::from(p),
        }
    }
}

impl From<[u16; 2]> for Padding {
    fn from(p: [u16; 2]) -> Self {
        Padding {
            top: f32::from(p[0]),
            right: f32::from(p[1]),
            bottom: f32::from(p[0]),
            left: f32::from(p[1]),
        }
    }
}

impl From<f32> for Padding {
    fn from(p: f32) -> Self {
        Padding {
            top: p,
            right: p,
            bottom: p,
            left: p,
        }
    }
}

impl From<[f32; 2]> for Padding {
    fn from(p: [f32; 2]) -> Self {
        Padding {
            top: p[0],
            right: p[1],
            bottom: p[0],
            left: p[1],
        }
    }
}

impl From<Padding> for Size {
    fn from(padding: Padding) -> Self {
        Self::new(padding.horizontal(), padding.vertical())
    }
}

impl From<Pixels> for Padding {
    fn from(pixels: Pixels) -> Self {
        Self::from(pixels.0)
    }
}
