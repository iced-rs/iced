/// An amount of space to pad for each side of a box
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
