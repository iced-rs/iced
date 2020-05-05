use smithay_client_toolkit::{
    window::{Frame, FrameRequest, State},
    reexports::client::{
        Attached, DispatchData,
        protocol::{wl_surface::WlSurface, wl_compositor::WlCompositor, wl_subcompositor::WlSubcompositor, wl_shm::WlShm, wl_seat::WlSeat}
    }
};

/// Zero 'smithay_client_toolkit::window::Frame'
#[derive(Debug)]
pub struct NoFrame {}

impl Frame for NoFrame {
    type Error = ::std::io::Error;
    type Config = ();
    fn init(
        _: &WlSurface,
        _: &Attached<WlCompositor>,
        _: &Attached<WlSubcompositor>,
        _: &Attached<WlShm>,
        _: Box<dyn FnMut(FrameRequest, u32, DispatchData)>,
    ) -> Result<Self, Self::Error> {
        Ok(Self{})
    }

    fn new_seat(&mut self, _: &Attached<WlSeat>) {}
    fn remove_seat(&mut self, _: &WlSeat) {}
    fn set_states(&mut self, _: &[State]) -> bool { false }
    fn set_hidden(&mut self, _: bool) {}
    fn set_resizable(&mut self, _: bool) {}
    fn resize(&mut self, _: (u32, u32)) {}
    fn redraw(&mut self) {}
    fn subtract_borders(&self, width: i32, height: i32) -> (i32, i32) { (width, height) }
    fn add_borders(&self, width: i32, height: i32) -> (i32, i32) { (width, height) }
    fn location(&self) -> (i32, i32) { (0, 0) }
    fn set_config(&mut self, _: Self::Config) {}
    fn set_title(&mut self, _: String) {}
}
