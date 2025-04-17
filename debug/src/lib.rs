pub use iced_core as core;
pub use iced_futures as futures;

use crate::core::theme;
use crate::core::window;
use crate::futures::Subscription;

pub use internal::Span;

use std::io;

#[derive(Debug, Clone, Copy)]
pub enum Primitive {
    Quad,
    Triangle,
    Shader,
    Image,
    Text,
}

#[derive(Debug, Clone, Copy)]
pub enum Command {
    RewindTo { message: usize },
}

pub fn init(name: &str) {
    internal::init(name);
}

pub fn toggle_comet() -> Result<(), io::Error> {
    internal::toggle_comet()
}

pub fn theme_changed(f: impl FnOnce() -> Option<theme::Palette>) {
    internal::theme_changed(f);
}

pub fn tasks_spawned(amount: usize) {
    internal::tasks_spawned(amount)
}

pub fn subscriptions_tracked(amount: usize) {
    internal::subscriptions_tracked(amount)
}

pub fn boot() -> Span {
    internal::boot()
}

pub fn update(message: &impl std::fmt::Debug) -> Span {
    internal::update(message)
}

pub fn view(window: window::Id) -> Span {
    internal::view(window)
}

pub fn layout(window: window::Id) -> Span {
    internal::layout(window)
}

pub fn interact(window: window::Id) -> Span {
    internal::interact(window)
}

pub fn draw(window: window::Id) -> Span {
    internal::draw(window)
}

pub fn prepare(primitive: Primitive) -> Span {
    internal::prepare(primitive)
}

pub fn render(primitive: Primitive) -> Span {
    internal::render(primitive)
}

pub fn present(window: window::Id) -> Span {
    internal::present(window)
}

pub fn time(name: impl Into<String>) -> Span {
    internal::time(name)
}

pub fn time_with<T>(name: impl Into<String>, f: impl FnOnce() -> T) -> T {
    let span = time(name);
    let result = f();
    span.finish();

    result
}

pub fn skip_next_timing() {
    internal::skip_next_timing();
}

pub fn commands() -> Subscription<Command> {
    internal::commands()
}

#[cfg(all(feature = "enable", not(target_arch = "wasm32")))]
mod internal {
    use crate::core::theme;
    use crate::core::time::Instant;
    use crate::core::window;
    use crate::futures::Subscription;
    use crate::futures::futures::Stream;
    use crate::{Command, Primitive};

    use iced_beacon as beacon;

    use beacon::client::{self, Client};
    use beacon::span;

    use std::io;
    use std::process;
    use std::sync::atomic::{self, AtomicBool, AtomicUsize};
    use std::sync::{LazyLock, RwLock};

    pub fn init(name: &str) {
        let name = name.split("::").next().unwrap_or(name);

        name.clone_into(&mut NAME.write().expect("Write application name"));
    }

    pub fn toggle_comet() -> Result<(), io::Error> {
        if BEACON.is_connected() {
            BEACON.quit();

            Ok(())
        } else {
            let _ = process::Command::new("iced_comet")
                .stdin(process::Stdio::null())
                .stdout(process::Stdio::null())
                .stderr(process::Stdio::null())
                .spawn()?;

            if let Some(palette) =
                LAST_PALETTE.read().expect("Read last palette").as_ref()
            {
                BEACON.log(client::Event::ThemeChanged(*palette));
            }

            Ok(())
        }
    }

    pub fn theme_changed(f: impl FnOnce() -> Option<theme::Palette>) {
        let Some(palette) = f() else {
            return;
        };

        if LAST_PALETTE.read().expect("Read last palette").as_ref()
            != Some(&palette)
        {
            BEACON.log(client::Event::ThemeChanged(palette));

            *LAST_PALETTE.write().expect("Write last palette") = Some(palette);
        }
    }

    pub fn tasks_spawned(amount: usize) {
        BEACON.log(client::Event::CommandsSpawned(amount));
    }

    pub fn subscriptions_tracked(amount: usize) {
        BEACON.log(client::Event::SubscriptionsTracked(amount));
    }

    pub fn boot() -> Span {
        span(span::Stage::Boot)
    }

    pub fn update(message: &impl std::fmt::Debug) -> Span {
        let span = span(span::Stage::Update);

        let number = LAST_UPDATE.fetch_add(1, atomic::Ordering::Relaxed);

        let start = Instant::now();
        let message = format!("{message:?}");
        let elapsed = start.elapsed();

        if elapsed.as_millis() >= 1 {
            log::warn!(
                "Slow `Debug` implementation of `Message` (took {elapsed:?})!"
            );
        }

        let message = if message.len() > 49 {
            format!("{}...", &message[..49])
        } else {
            message
        };

        BEACON.log(client::Event::MessageLogged { number, message });

        span
    }

    pub fn view(window: window::Id) -> Span {
        span(span::Stage::View(window))
    }

    pub fn layout(window: window::Id) -> Span {
        span(span::Stage::Layout(window))
    }

    pub fn interact(window: window::Id) -> Span {
        span(span::Stage::Interact(window))
    }

    pub fn draw(window: window::Id) -> Span {
        span(span::Stage::Draw(window))
    }

    pub fn prepare(primitive: Primitive) -> Span {
        span(span::Stage::Prepare(to_primitive(primitive)))
    }

    pub fn render(primitive: Primitive) -> Span {
        span(span::Stage::Render(to_primitive(primitive)))
    }

    pub fn present(window: window::Id) -> Span {
        span(span::Stage::Present(window))
    }

    pub fn time(name: impl Into<String>) -> Span {
        span(span::Stage::Custom(name.into()))
    }

    pub fn skip_next_timing() {
        SKIP_NEXT_SPAN.store(true, atomic::Ordering::Relaxed);
    }

    pub fn commands() -> Subscription<Command> {
        fn listen_for_commands() -> impl Stream<Item = Command> {
            use crate::futures::futures::stream;

            stream::unfold(BEACON.subscribe(), async move |mut receiver| {
                let command = match receiver.recv().await? {
                    client::Command::RewindTo { message } => {
                        Command::RewindTo { message }
                    }
                };

                Some((command, receiver))
            })
        }

        Subscription::run(listen_for_commands)
    }

    fn span(span: span::Stage) -> Span {
        BEACON.log(client::Event::SpanStarted(span.clone()));

        Span {
            span,
            start: Instant::now(),
        }
    }

    fn to_primitive(primitive: Primitive) -> span::Primitive {
        match primitive {
            Primitive::Quad => span::Primitive::Quad,
            Primitive::Triangle => span::Primitive::Triangle,
            Primitive::Shader => span::Primitive::Shader,
            Primitive::Text => span::Primitive::Text,
            Primitive::Image => span::Primitive::Image,
        }
    }

    #[derive(Debug)]
    pub struct Span {
        span: span::Stage,
        start: Instant,
    }

    impl Span {
        pub fn finish(self) {
            if SKIP_NEXT_SPAN.fetch_and(false, atomic::Ordering::Relaxed) {
                return;
            }

            BEACON.log(client::Event::SpanFinished(
                self.span,
                self.start.elapsed(),
            ));
        }
    }

    static BEACON: LazyLock<Client> = LazyLock::new(|| {
        client::connect(NAME.read().expect("Read application name").to_owned())
    });

    static NAME: RwLock<String> = RwLock::new(String::new());
    static LAST_UPDATE: AtomicUsize = AtomicUsize::new(0);
    static LAST_PALETTE: RwLock<Option<theme::Palette>> = RwLock::new(None);
    static SKIP_NEXT_SPAN: AtomicBool = AtomicBool::new(false);
}

#[cfg(any(not(feature = "enable"), target_arch = "wasm32"))]
mod internal {
    use crate::core::theme;
    use crate::core::window;
    use crate::futures::Subscription;
    use crate::{Command, Primitive};

    use std::io;

    pub fn init(_name: &str) {}

    pub fn toggle_comet() -> Result<(), io::Error> {
        Ok(())
    }

    pub fn theme_changed(_f: impl FnOnce() -> Option<theme::Palette>) {}

    pub fn tasks_spawned(_amount: usize) {}

    pub fn subscriptions_tracked(_amount: usize) {}

    pub fn boot() -> Span {
        Span
    }

    pub fn update(_message: &impl std::fmt::Debug) -> Span {
        Span
    }

    pub fn view(_window: window::Id) -> Span {
        Span
    }

    pub fn layout(_window: window::Id) -> Span {
        Span
    }

    pub fn interact(_window: window::Id) -> Span {
        Span
    }

    pub fn draw(_window: window::Id) -> Span {
        Span
    }

    pub fn prepare(_primitive: Primitive) -> Span {
        Span
    }

    pub fn render(_primitive: Primitive) -> Span {
        Span
    }

    pub fn present(_window: window::Id) -> Span {
        Span
    }

    pub fn time(_name: impl Into<String>) -> Span {
        Span
    }

    pub fn skip_next_timing() {}

    pub fn commands() -> Subscription<Command> {
        Subscription::none()
    }

    #[derive(Debug)]
    pub struct Span;

    impl Span {
        pub fn finish(self) {}
    }
}
