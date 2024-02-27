pub use iced_core as core;
pub use iced_style as style;

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

pub fn view_time() -> Timer {
    internal::view_time()
}

pub fn layout_time() -> Timer {
    internal::layout_time()
}

pub fn interact_time() -> Timer {
    internal::interact_time()
}

pub fn draw_time() -> Timer {
    internal::draw_time()
}

pub fn render_time() -> Timer {
    internal::render_time()
}

pub fn time(name: impl AsRef<str>) -> Timer {
    internal::time(name)
}

#[cfg(feature = "enable")]
mod internal {
    use crate::core::time::Instant;
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

    pub fn view_time() -> Timer {
        timer(timing::Stage::View)
    }

    pub fn layout_time() -> Timer {
        timer(timing::Stage::Layout)
    }

    pub fn interact_time() -> Timer {
        timer(timing::Stage::Interact)
    }

    pub fn draw_time() -> Timer {
        timer(timing::Stage::Draw)
    }

    pub fn render_time() -> Timer {
        timer(timing::Stage::Render)
    }

    pub fn time(name: impl AsRef<str>) -> Timer {
        timer(timing::Stage::Custom(name.as_ref().to_owned()))
    }

    fn timer(stage: timing::Stage) -> Timer {
        Timer {
            stage,
            start: Instant::now(),
        }
    }

    #[derive(Debug)]
    pub struct Timer {
        stage: timing::Stage,
        start: Instant,
    }

    impl Timer {
        pub fn finish(self) {
            lock().sentinel.report_timing(Timing {
                stage: self.stage,
                duration: self.start.elapsed(),
            });
        }
    }

    #[derive(Debug)]
    struct Debug {
        sentinel: Client,
        last_palette: Option<theme::Palette>,
    }

    fn lock() -> MutexGuard<'static, Debug> {
        static DEBUG: Lazy<Mutex<Debug>> = Lazy::new(|| {
            Mutex::new(Debug {
                sentinel: client::connect(),
                last_palette: None,
            })
        });

        DEBUG.lock().expect("Acquire debug lock")
    }
}

#[cfg(not(feature = "enable"))]
mod internal {
    use crate::style::theme;

    pub fn theme_changed(_palette: theme::Palette) {}

    pub fn boot_time() -> Timer {
        Timer
    }

    pub fn update_time() -> Timer {
        Timer
    }

    pub fn view_time() -> Timer {
        Timer
    }

    pub fn layout_time() -> Timer {
        Timer
    }

    pub fn interact_time() -> Timer {
        Timer
    }

    pub fn draw_time() -> Timer {
        Timer
    }

    pub fn render_time() -> Timer {
        Timer
    }

    pub fn time(_name: impl AsRef<str>) -> Timer {
        Timer
    }

    #[derive(Debug)]
    pub struct Timer;

    impl Timer {
        pub fn finish(self) {}
    }
}
