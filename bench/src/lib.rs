pub mod glow;
pub mod wgpu;

pub enum Msg {}

pub fn render_widget<
    R: iced_native::Renderer<Theme = iced::Theme>,
    W: iced_native::Widget<Msg, R>,
>(
    widget: &W,
    renderer: &mut R,
) {
    let size = iced::Size::new(1024.0, 1024.0);
    let node = iced_native::layout::Node::new(size);
    let layout = iced_native::Layout::new(&node);
    widget.draw(
        &iced_native::widget::Tree::empty(),
        renderer,
        &iced::Theme::Light,
        &Default::default(),
        layout,
        iced::Point::new(0.0, 0.0),
        &iced::Rectangle::with_size(size),
    );
}

pub trait Bench {
    type Backend: iced_graphics::Backend;
    type RenderState;
    const BACKEND_NAME: &'static str;

    fn clear(&self) -> Self::RenderState;
    fn present(&mut self, state: Self::RenderState);
    fn read_pixels(&self) -> image_rs::RgbaImage;
    fn size(&self) -> (u32, u32);
    fn renderer(
        &mut self,
    ) -> &mut iced_graphics::Renderer<Self::Backend, iced::Theme>;
}
