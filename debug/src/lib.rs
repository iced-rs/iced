pub use iced_core as core;
pub use iced_style as style;

use crate::core::window;
use crate::style::theme;

pub use internal::Timer;

pub fn open_axe() {}

pub fn log_message(_message: &impl std::fmt::Debug) {}

pub fn theme_changed(palette: theme::Palette) {
    internal::theme_changed(palette);
}

pub fn boot_time() -> Timer {
    internal::boot_time()
}

pub fn update_time() -> Timer {
    internal::update_time()
}

pub fn view_time(window: window::Id) -> Timer {
    internal::view_time(window)
}

pub fn layout_time(window: window::Id) -> Timer {
    internal::layout_time(window)
}

pub fn interact_time(window: window::Id) -> Timer {
    internal::interact_time(window)
}

pub fn draw_time(window: window::Id) -> Timer {
    internal::draw_time(window)
}

pub fn render_time(window: window::Id) -> Timer {
    internal::render_time(window)
}

pub fn time(window: window::Id, name: impl AsRef<str>) -> Timer {
    internal::time(window, name)
}

pub fn skip_next_timing() {
    internal::skip_next_timing();
}

#[cfg(feature = "enable")]
mod internal {
    use crate::core::time::{Instant, SystemTime};
    use crate::core::window;
    use crate::style::theme;

    use iced_sentinel::client::{self, Client};
    use iced_sentinel::timing::{self, Timing};

    use once_cell::sync::Lazy;
    use std::sync::{Mutex, MutexGuard};

    pub fn theme_changed(palette: theme::Palette) {
        let mut debug = lock();

        if debug.last_palette.as_ref() != Some(&palette) {
            debug.sentinel.report_theme_change(palette);

            debug.last_palette = Some(palette);
        }
    }

    pub fn boot_time() -> Timer {
        timer(timing::Stage::Boot)
    }

    pub fn update_time() -> Timer {
        timer(timing::Stage::Update)
    }

    pub fn view_time(window: window::Id) -> Timer {
        timer(timing::Stage::View(window))
    }

    pub fn layout_time(window: window::Id) -> Timer {
        timer(timing::Stage::Layout(window))
    }

    pub fn interact_time(window: window::Id) -> Timer {
        timer(timing::Stage::Interact(window))
    }

    pub fn draw_time(window: window::Id) -> Timer {
        timer(timing::Stage::Draw(window))
    }

    pub fn render_time(window: window::Id) -> Timer {
        timer(timing::Stage::Render(window))
    }

    pub fn time(window: window::Id, name: impl AsRef<str>) -> Timer {
        timer(timing::Stage::Custom(window, name.as_ref().to_owned()))
    }

    pub fn skip_next_timing() {
        lock().skip_next_timing = true;
    }

    fn timer(stage: timing::Stage) -> Timer {
        Timer {
            stage,
            start: Instant::now(),
            start_system_time: SystemTime::now(),
        }
    }

    #[derive(Debug)]
    pub struct Timer {
        stage: timing::Stage,
        start: Instant,
        start_system_time: SystemTime,
    }

    impl Timer {
        pub fn finish(self) {
            let mut debug = lock();

            if debug.skip_next_timing {
                debug.skip_next_timing = false;
                return;
            }

            debug.sentinel.report_timing(Timing {
                stage: self.stage,
                start: self.start_system_time,
                duration: self.start.elapsed(),
            });
        }
    }

    #[derive(Debug)]
    struct Debug {
        sentinel: Client,
        last_palette: Option<theme::Palette>,
        skip_next_timing: bool,
    }

    fn lock() -> MutexGuard<'static, Debug> {
        static DEBUG: Lazy<Mutex<Debug>> = Lazy::new(|| {
            Mutex::new(Debug {
                sentinel: client::connect(),
                last_palette: None,
                skip_next_timing: false,
            })
        });

        DEBUG.lock().expect("Acquire debug lock")
    }
}

#[cfg(not(feature = "enable"))]
mod internal {
    use crate::core::window;
    use crate::style::theme;

    pub fn theme_changed(_palette: theme::Palette) {}

    pub fn boot_time() -> Timer {
        Timer
    }

    pub fn update_time() -> Timer {
        Timer
    }

    pub fn view_time(_window: window::Id) -> Timer {
        Timer
    }

    pub fn layout_time(_window: window::Id) -> Timer {
        Timer
    }

    pub fn interact_time(_window: window::Id) -> Timer {
        Timer
    }

    pub fn draw_time(_window: window::Id) -> Timer {
        Timer
    }

    pub fn render_time(_window: window::Id) -> Timer {
        Timer
    }

    pub fn time(_window: window::Id, _name: impl AsRef<str>) -> Timer {
        Timer
    }

    pub fn skip_next_timing() {}

    #[derive(Debug)]
    pub struct Timer;

    impl Timer {
        pub fn finish(self) {}
    }
}
