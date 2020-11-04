use crate::Size;

pub trait Compositor {
    fn window(&self) -> &winit::window::Window;

    fn resize(&self, new_size: Size<u32>);
}
