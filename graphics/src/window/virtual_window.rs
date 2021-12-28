use iced_native::screenshot::Screenshot;

/// Backend agnostic trait for application logic to interact with Compositor logic
pub trait VirtualCompositor {
    /// Reads the framebuffer pixels on the provided region into the provided buffer.
    fn read(&self) -> Option<Screenshot>;

    /// Returns true if this virtual compositor has to re-render to a buffer
    fn requires_rerender(&self) -> bool {
        false
    }

    /// Queues a screenshot. This used by Compositors (wgpu) that require a re-render of the entire
    /// screen in order to do a screenshot
    fn queue_screenshot(
        &mut self,
        _generator: Box<dyn Fn(Option<Screenshot>)>,
    ) {
    }

    /// Checks 
    fn is_screenshot_queued(&self) -> bool {
        false
    }

    ///dequeues a queued
    fn dequeue_screenshot(&mut self) {}
}
