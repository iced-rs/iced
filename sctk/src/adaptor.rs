use accesskit::{kurbo::Rect, ActionHandler, TreeUpdate};
use accesskit_unix::Adapter as UnixAdapter;
use winit::window::Window;

pub struct Adapter {
    adapter: Option<UnixAdapter>,
}

impl Adapter {
    pub fn new(
        _: &Window,
        source: impl 'static + FnOnce() -> TreeUpdate,
        action_handler: Box<dyn ActionHandler>,
    ) -> Self {
        let adapter = UnixAdapter::new(
            String::new(),
            String::new(),
            String::new(),
            source,
            action_handler,
        );
        Self { adapter }
    }

    pub fn set_root_window_bounds(&self, outer: Rect, inner: Rect) {
        if let Some(adapter) = &self.adapter {
            adapter.set_root_window_bounds(outer, inner);
        }
    }

    pub fn update(&self, update: TreeUpdate) {
        if let Some(adapter) = &self.adapter {
            adapter.update(update);
        }
    }

    pub fn update_if_active(&self, updater: impl FnOnce() -> TreeUpdate) {
        if let Some(adapter) = &self.adapter {
            adapter.update(updater());
        }
    }
}
