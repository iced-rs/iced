//! A rectangle with certain styled properties.

use bytemuck::{Pod, Zeroable};

/// The properties of a quad.
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Properties {
    /// The position of the quad.
    pub position: [f32; 2],

    /// The size of the quad.
    pub size: [f32; 2],

    /// The border color of the quad, in __linear RGB__.
    pub border_color: [f32; 4],

    /// The border radii of the quad.
    pub border_radius: [f32; 4],

    /// The border width of the quad.
    pub border_width: f32,
}

/// A quad filled with a solid color.
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Solid {
    /// The background color data of the quad.
    pub color: [f32; 4],

    /// The [`Properties`] of the quad.
    pub properties: Properties,
}

/// A quad filled with interpolated colors.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Gradient {
    /// The background gradient data of the quad.
    pub gradient: [f32; 44],

    /// The [`Properties`] of the quad.
    pub properties: Properties,
}

#[allow(unsafe_code)]
unsafe impl Pod for Gradient {}

#[allow(unsafe_code)]
unsafe impl Zeroable for Gradient {}
