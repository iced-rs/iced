//! The definition of an iced program.
pub use iced_graphics as graphics;
pub use iced_runtime as runtime;
pub use iced_runtime::core;
pub use iced_runtime::futures;

pub mod message;

mod preset;

pub use preset::Preset;

use crate::core::renderer;
use crate::core::text;
use crate::core::theme;
use crate::core::window;
use crate::core::{Element, Font, Settings};
use crate::futures::{Executor, Subscription};
use crate::graphics::compositor;
use crate::runtime::Task;

/// An interactive, native, cross-platform, multi-windowed application.
///
/// A [`Program`] can execute asynchronous actions by returning a
/// [`Task`] in some of its methods.
#[allow(missing_docs)]
pub trait Program: Sized {
    /// The state of the program.
    type State;

    /// The message of the program.
    type Message: Send + 'static;

    /// The theme of the program.
    type Theme: theme::Base;

    /// The renderer of the program.
    type Renderer: Renderer;

    /// The executor of the program.
    type Executor: Executor;

    /// Returns the unique name of the [`Program`].
    fn name() -> &'static str;

    fn settings(&self) -> Settings;

    fn window(&self) -> Option<window::Settings>;

    fn boot(&self) -> (Self::State, Task<Self::Message>);

    fn update(&self, state: &mut Self::State, message: Self::Message) -> Task<Self::Message>;

    fn view<'a>(
        &self,
        state: &'a Self::State,
        window: window::Id,
    ) -> Element<'a, Self::Message, Self::Theme, Self::Renderer>;

    fn title(&self, _state: &Self::State, _window: window::Id) -> String {
        let mut title = String::new();

        for (i, part) in Self::name().split("_").enumerate() {
            use std::borrow::Cow;

            let part = match part {
                "a" | "an" | "of" | "in" | "and" => Cow::Borrowed(part),
                _ => {
                    let mut part = part.to_owned();

                    if let Some(first_letter) = part.get_mut(0..1) {
                        first_letter.make_ascii_uppercase();
                    }

                    Cow::Owned(part)
                }
            };

            if i > 0 {
                title.push(' ');
            }

            title.push_str(&part);
        }

        format!("{title} - Iced")
    }

    fn subscription(&self, _state: &Self::State) -> Subscription<Self::Message> {
        Subscription::none()
    }

    fn theme(&self, _state: &Self::State, _window: window::Id) -> Option<Self::Theme> {
        None
    }

    fn style(&self, _state: &Self::State, theme: &Self::Theme) -> theme::Style {
        theme::Base::base(theme)
    }

    fn scale_factor(&self, _state: &Self::State, _window: window::Id) -> f32 {
        1.0
    }

    fn presets(&self) -> &[Preset<Self::State, Self::Message>] {
        &[]
    }
}

/// Decorates a [`Program`] with the given subscription function.
pub fn with_subscription<P: Program>(
    program: P,
    f: impl Fn(&P::State) -> Subscription<P::Message>,
) -> impl Program<State = P::State, Message = P::Message, Theme = P::Theme> {
    struct WithSubscription<P, F> {
        program: P,
        subscription: F,
    }

    impl<P: Program, F> Program for WithSubscription<P, F>
    where
        F: Fn(&P::State) -> Subscription<P::Message>,
    {
        type State = P::State;
        type Message = P::Message;
        type Theme = P::Theme;
        type Renderer = P::Renderer;
        type Executor = P::Executor;

        #[inline]
        fn subscription(&self, state: &Self::State) -> Subscription<Self::Message> {
            (self.subscription)(state)
        }

        #[inline]
        fn name() -> &'static str {
            P::name()
        }

        #[inline]
        fn settings(&self) -> Settings {
            self.program.settings()
        }

        #[inline]
        fn window(&self) -> Option<window::Settings> {
            self.program.window()
        }

        #[inline]
        fn boot(&self) -> (Self::State, Task<Self::Message>) {
            self.program.boot()
        }

        #[inline]
        fn update(&self, state: &mut Self::State, message: Self::Message) -> Task<Self::Message> {
            self.program.update(state, message)
        }

        #[inline]
        fn view<'a>(
            &self,
            state: &'a Self::State,
            window: window::Id,
        ) -> Element<'a, Self::Message, Self::Theme, Self::Renderer> {
            self.program.view(state, window)
        }

        #[inline]
        fn title(&self, state: &Self::State, window: window::Id) -> String {
            self.program.title(state, window)
        }

        #[inline]
        fn theme(&self, state: &Self::State, window: window::Id) -> Option<Self::Theme> {
            self.program.theme(state, window)
        }

        #[inline]
        fn style(&self, state: &Self::State, theme: &Self::Theme) -> theme::Style {
            self.program.style(state, theme)
        }

        #[inline]
        fn scale_factor(&self, state: &Self::State, window: window::Id) -> f32 {
            self.program.scale_factor(state, window)
        }
    }

    WithSubscription {
        program,
        subscription: f,
    }
}

/// Decorates a [`Program`] with the given style function.
pub fn with_style<P: Program>(
    program: P,
    f: impl Fn(&P::State, &P::Theme) -> theme::Style,
) -> impl Program<State = P::State, Message = P::Message, Theme = P::Theme> {
    struct WithStyle<P, F> {
        program: P,
        style: F,
    }

    impl<P: Program, F> Program for WithStyle<P, F>
    where
        F: Fn(&P::State, &P::Theme) -> theme::Style,
    {
        type State = P::State;
        type Message = P::Message;
        type Theme = P::Theme;
        type Renderer = P::Renderer;
        type Executor = P::Executor;

        #[inline]
        fn style(&self, state: &Self::State, theme: &Self::Theme) -> theme::Style {
            (self.style)(state, theme)
        }

        #[inline]
        fn name() -> &'static str {
            P::name()
        }

        #[inline]
        fn settings(&self) -> Settings {
            self.program.settings()
        }

        #[inline]
        fn window(&self) -> Option<window::Settings> {
            self.program.window()
        }

        #[inline]
        fn boot(&self) -> (Self::State, Task<Self::Message>) {
            self.program.boot()
        }

        #[inline]
        fn title(&self, state: &Self::State, window: window::Id) -> String {
            self.program.title(state, window)
        }

        #[inline]
        fn update(&self, state: &mut Self::State, message: Self::Message) -> Task<Self::Message> {
            self.program.update(state, message)
        }

        #[inline]
        fn view<'a>(
            &self,
            state: &'a Self::State,
            window: window::Id,
        ) -> Element<'a, Self::Message, Self::Theme, Self::Renderer> {
            self.program.view(state, window)
        }

        #[inline]
        fn subscription(&self, state: &Self::State) -> Subscription<Self::Message> {
            self.program.subscription(state)
        }

        #[inline]
        fn theme(&self, state: &Self::State, window: window::Id) -> Option<Self::Theme> {
            self.program.theme(state, window)
        }

        #[inline]
        fn scale_factor(&self, state: &Self::State, window: window::Id) -> f32 {
            self.program.scale_factor(state, window)
        }
    }

    WithStyle { program, style: f }
}

/// Decorates a [`Program`] with the given executor function.
pub fn with_executor<P: Program, E: Executor>(
    program: P,
) -> impl Program<State = P::State, Message = P::Message, Theme = P::Theme> {
    use std::marker::PhantomData;

    struct WithExecutor<P, E> {
        program: P,
        executor: PhantomData<E>,
    }

    impl<P: Program, E> Program for WithExecutor<P, E>
    where
        E: Executor,
    {
        type State = P::State;
        type Message = P::Message;
        type Theme = P::Theme;
        type Renderer = P::Renderer;
        type Executor = E;

        #[inline]
        fn title(&self, state: &Self::State, window: window::Id) -> String {
            self.program.title(state, window)
        }

        #[inline]
        fn name() -> &'static str {
            P::name()
        }

        #[inline]
        fn settings(&self) -> Settings {
            self.program.settings()
        }

        #[inline]
        fn window(&self) -> Option<window::Settings> {
            self.program.window()
        }

        #[inline]
        fn boot(&self) -> (Self::State, Task<Self::Message>) {
            self.program.boot()
        }

        #[inline]
        fn update(&self, state: &mut Self::State, message: Self::Message) -> Task<Self::Message> {
            self.program.update(state, message)
        }

        #[inline]
        fn view<'a>(
            &self,
            state: &'a Self::State,
            window: window::Id,
        ) -> Element<'a, Self::Message, Self::Theme, Self::Renderer> {
            self.program.view(state, window)
        }

        #[inline]
        fn subscription(&self, state: &Self::State) -> Subscription<Self::Message> {
            self.program.subscription(state)
        }

        #[inline]
        fn theme(&self, state: &Self::State, window: window::Id) -> Option<Self::Theme> {
            self.program.theme(state, window)
        }

        #[inline]
        fn style(&self, state: &Self::State, theme: &Self::Theme) -> theme::Style {
            self.program.style(state, theme)
        }

        #[inline]
        fn scale_factor(&self, state: &Self::State, window: window::Id) -> f32 {
            self.program.scale_factor(state, window)
        }
    }

    WithExecutor {
        program,
        executor: PhantomData::<E>,
    }
}

/// The renderer of some [`Program`].
pub trait Renderer: text::Renderer<Font = Font> + compositor::Default + renderer::Headless {}

impl<T> Renderer for T where
    T: text::Renderer<Font = Font> + compositor::Default + renderer::Headless
{
}

/// A particular instance of a running [`Program`].
pub struct Instance<P: Program> {
    program: P,
    state: P::State,
}

impl<P: Program> Instance<P> {
    /// Creates a new [`Instance`] of the given [`Program`].
    pub fn new(program: P) -> (Self, Task<P::Message>) {
        let (state, task) = program.boot();

        (Self { program, state }, task)
    }

    /// Returns the current title of the [`Instance`].
    #[inline]
    pub fn title(&self, window: window::Id) -> String {
        self.program.title(&self.state, window)
    }

    /// Processes the given message and updates the [`Instance`].
    #[inline]
    pub fn update(&mut self, message: P::Message) -> Task<P::Message> {
        self.program.update(&mut self.state, message)
    }

    /// Produces the current widget tree of the [`Instance`].
    #[inline]
    pub fn view(&self, window: window::Id) -> Element<'_, P::Message, P::Theme, P::Renderer> {
        self.program.view(&self.state, window)
    }

    /// Returns the current [`Subscription`] of the [`Instance`].
    #[inline]
    pub fn subscription(&self) -> Subscription<P::Message> {
        self.program.subscription(&self.state)
    }

    /// Returns the current theme of the [`Instance`].
    #[inline]
    pub fn theme(&self, window: window::Id) -> Option<P::Theme> {
        self.program.theme(&self.state, window)
    }

    /// Returns the current [`theme::Style`] of the [`Instance`].
    #[inline]
    pub fn style(&self, theme: &P::Theme) -> theme::Style {
        self.program.style(&self.state, theme)
    }

    /// Returns the current scale factor of the [`Instance`].
    #[inline]
    pub fn scale_factor(&self, window: window::Id) -> f32 {
        self.program.scale_factor(&self.state, window)
    }
}
