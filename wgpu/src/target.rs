use crate::Viewport;

/// A rendering target.
#[derive(Debug)]
pub struct Target<'a> {
    /// The texture where graphics will be rendered.
    pub texture: &'a wgpu::TextureView,

    /// The viewport of the target.
    ///
    /// Most of the time, you will want this to match the dimensions of the
    /// texture.
    pub viewport: &'a Viewport,
}
