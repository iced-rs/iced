use crate::Viewport;

/// A rendering target.
#[derive(Debug)]
pub struct Target<'a> {
    pub texture: &'a wgpu::TextureView,
    pub viewport: &'a Viewport,
}
