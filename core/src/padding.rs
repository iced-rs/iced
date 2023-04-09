/// An amount of space to pad for each side of a box
///
/// You can leverage the `From` trait to build [`Padding`] conveniently:
///
/// ```
/// # use iced_core::Padding;
/// #
/// let padding = Padding::from(20);              // 20px on all sides
/// let padding = Padding::from([10, 20]);        // top/bottom, left/right
/// let padding = Padding::from([5, 10, 15, 20]); // top, right, bottom, left
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
/// let widget = Widget::new().padding([5, 10, 15, 20]); // top, right, bottom, left
/// ```
#[derive(Debug, Hash, Copy, Clone)]
pub struct Padding {
    /// Top padding
    pub top: u16,
    /// Right padding
    pub right: u16,
    /// Bottom padding
    pub bottom: u16,
    /// Left padding
    pub left: u16,
}

impl Padding {
    /// Padding of zero
    pub const ZERO: Padding = Padding {
        top: 0,
        right: 0,
        bottom: 0,
        left: 0,
    };

    /// Create a Padding that is equal on all sides
    pub const fn new(padding: u16) -> Padding {
        Padding {
            top: padding,
            right: padding,
            bottom: padding,
            left: padding,
        }
    }

    /// Returns the total amount of vertical [`Padding`].
    pub fn vertical(self) -> u16 {
        self.top + self.bottom
    }

    /// Returns the total amount of horizontal [`Padding`].
    pub fn horizontal(self) -> u16 {
        self.left + self.right
    }
}

impl std::convert::From<u16> for Padding {
    fn from(p: u16) -> Self {
        Padding {
            top: p,
            right: p,
            bottom: p,
            left: p,
        }
    }
}

impl std::convert::From<[u16; 2]> for Padding {
    fn from(p: [u16; 2]) -> Self {
        Padding {
            top: p[0],
            right: p[1],
            bottom: p[0],
            left: p[1],
        }
    }
}

impl std::convert::From<[u16; 4]> for Padding {
    fn from(p: [u16; 4]) -> Self {
        Padding {
            top: p[0],
            right: p[1],
            bottom: p[2],
            left: p[3],
        }
    }
}
