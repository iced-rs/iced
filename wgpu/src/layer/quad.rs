//! A rectangle with certain styled properties.
use crate::graphics::gradient;
use bytemuck::{Pod, Zeroable};

/// The properties of a quad.
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Quad {
    /// The position of the [`Quad`].
    pub position: [f32; 2],

    /// The size of the [`Quad`].
    pub size: [f32; 2],

    /// The border color of the [`Quad`], in __linear RGB__.
    pub border_color: [f32; 4],

    /// The border radii of the [`Quad`].
    pub border_radius: [f32; 4],

    /// The border width of the [`Quad`].
    pub border_width: f32,
}

/// A quad filled with a solid color.
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Solid {
    /// The background color data of the quad.
    pub color: [f32; 4],

    /// The [`Quad`] data of the [`Solid`].
    pub quad: Quad,
}

/// A quad filled with interpolated colors.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Gradient {
    /// The background gradient data of the quad.
    pub gradient: gradient::Packed,

    /// The [`Quad`] data of the [`Gradient`].
    pub quad: Quad,
}

#[allow(unsafe_code)]
unsafe impl Pod for Gradient {}

#[allow(unsafe_code)]
unsafe impl Zeroable for Gradient {}

#[derive(Debug, Copy, Clone)]
/// The identifier of a quad, used for ordering.
pub enum Order {
    /// A solid quad
    Solid,
    /// A gradient quad
    Gradient,
}
