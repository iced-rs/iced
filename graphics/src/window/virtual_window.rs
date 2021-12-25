use iced_native::screenshot::Screenshot;

/// Backend agnostic trait for application logic to interact with Compositor logic
pub trait VirtualCompositor {
    /// Reads the framebuffer pixels on the provided region into the provided buffer.
    fn read(&self) -> Option<Screenshot>;

}
