//! A rectangle with certain styled properties.
use crate::core::{Background, Rectangle};
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

/// A group of [`Quad`]s rendered together.
#[derive(Default, Debug)]
pub struct Layer {
    /// The solid quads of the [`Layer`].
    solids: Vec<Solid>,

    /// The gradient quads of the [`Layer`].
    gradients: Vec<Gradient>,

    /// The quad order of the [`Layer`]; stored as a tuple of the quad type & its count.
    order: Vec<(Order, usize)>,

    /// The last index of quad ordering.
    index: usize,
}

impl Layer {
    /// Returns true if there are no quads of any type in [`Quads`].
    pub fn is_empty(&self) -> bool {
        self.solids.is_empty() && self.gradients.is_empty()
    }

    /// The [`Solid`] quads of the [`Layer`].
    pub fn solids(&self) -> &[Solid] {
        &self.solids
    }

    /// The [`Gradient`] quads of the [`Layer`].
    pub fn gradients(&self) -> &[Gradient] {
        &self.gradients
    }

    /// The order of quads within the [`Layer`], grouped by (type, count) for rendering in batches.
    pub fn ordering(&self) -> &[(Order, usize)] {
        &self.order
    }

    /// Adds a [`Quad`] with the provided `Background` type to the quad [`Layer`].
    pub fn add(&mut self, quad: Quad, background: &Background) {
        let quad_order = match background {
            Background::Color(color) => {
                self.solids.push(Solid {
                    color: color.into_linear(),
                    quad,
                });

                Order::Solid
            }
            Background::Gradient(gradient) => {
                let quad = Gradient {
                    gradient: gradient::pack(
                        gradient,
                        Rectangle::new(quad.position.into(), quad.size.into()),
                    ),
                    quad,
                };

                self.gradients.push(quad);
                Order::Gradient
            }
        };

        match (self.order.get_mut(self.index), quad_order) {
            (Some((quad_order, count)), Order::Solid) => match quad_order {
                Order::Solid => {
                    *count += 1;
                }
                Order::Gradient => {
                    self.order.push((Order::Solid, 1));
                    self.index += 1;
                }
            },
            (Some((quad_order, count)), Order::Gradient) => match quad_order {
                Order::Solid => {
                    self.order.push((Order::Gradient, 1));
                    self.index += 1;
                }
                Order::Gradient => {
                    *count += 1;
                }
            },
            (None, _) => {
                self.order.push((quad_order, 1));
            }
        }
    }
}
