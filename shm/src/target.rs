/// A rendering target.
#[derive(Debug)]
pub struct Target<'t> {
    /// The texture where graphics will be rendered.
    pub texture: &'t mut framework::widget::Target<'t>,

    /// The viewport of the target.
    ///
    /// Most of the time, you will want this to match the dimensions of the
    /// texture.
    pub viewport: &'t crate::Viewport,
}
