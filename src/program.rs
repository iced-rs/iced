use crate::core::text;
use crate::graphics::compositor;
use crate::shell;
use crate::window;
use crate::{Element, Executor, Result, Settings, Subscription, Task};

pub use crate::shell::program::{Appearance, DefaultStyle};

/// The internal definition of a [`Program`].
///
/// You should not need to implement this trait directly. Instead, use the
/// methods available in the [`Program`] struct.
#[allow(missing_docs)]
pub trait Program: Sized {
    /// The state of the program.
    type State;

    /// The message of the program.
    type Message: Send + std::fmt::Debug + 'static;

    /// The theme of the program.
    type Theme: Default + DefaultStyle;

    /// The renderer of the program.
    type Renderer: Renderer;

    /// The executor of the program.
    type Executor: Executor;

    fn update(
        &self,
        state: &mut Self::State,
        message: Self::Message,
    ) -> Task<Self::Message>;

    fn view<'a>(
        &self,
        state: &'a Self::State,
        window: window::Id,
    ) -> Element<'a, Self::Message, Self::Theme, Self::Renderer>;

    fn title(&self, _state: &Self::State, _window: window::Id) -> String {
        String::from("A cool iced application!")
    }

    fn subscription(
        &self,
        _state: &Self::State,
    ) -> Subscription<Self::Message> {
        Subscription::none()
    }

    fn theme(&self, _state: &Self::State, _window: window::Id) -> Self::Theme {
        Self::Theme::default()
    }

    fn style(&self, _state: &Self::State, theme: &Self::Theme) -> Appearance {
        DefaultStyle::default_style(theme)
    }

    fn scale_factor(&self, _state: &Self::State, _window: window::Id) -> f64 {
        1.0
    }

    /// Runs the [`Program`].
    ///
    /// The state of the [`Program`] must implement [`Default`].
    /// If your state does not implement [`Default`], use [`run_with`]
    /// instead.
    ///
    /// [`run_with`]: Self::run_with
    fn run(
        self,
        settings: Settings,
        window_settings: Option<window::Settings>,
    ) -> Result
    where
        Self: 'static,
        Self::State: Default,
    {
        self.run_with(settings, window_settings, || {
            (Self::State::default(), Task::none())
        })
    }

    /// Runs the [`Program`] with the given [`Settings`] and a closure that creates the initial state.
    fn run_with<I>(
        self,
        settings: Settings,
        window_settings: Option<window::Settings>,
        initialize: I,
    ) -> Result
    where
        Self: 'static,
        I: Fn() -> (Self::State, Task<Self::Message>) + Clone + 'static,
    {
        use std::marker::PhantomData;

        struct Instance<P: Program, I> {
            program: P,
            state: P::State,
            _initialize: PhantomData<I>,
        }

        impl<P: Program, I: Fn() -> (P::State, Task<P::Message>)> shell::Program
            for Instance<P, I>
        {
            type Message = P::Message;
            type Theme = P::Theme;
            type Renderer = P::Renderer;
            type Flags = (P, I);
            type Executor = P::Executor;

            fn new(
                (program, initialize): Self::Flags,
            ) -> (Self, Task<Self::Message>) {
                let (state, task) = initialize();

                (
                    Self {
                        program,
                        state,
                        _initialize: PhantomData,
                    },
                    task,
                )
            }

            fn title(&self, window: window::Id) -> String {
                self.program.title(&self.state, window)
            }

            fn update(
                &mut self,
                message: Self::Message,
            ) -> Task<Self::Message> {
                self.program.update(&mut self.state, message)
            }

            fn view(
                &self,
                window: window::Id,
            ) -> crate::Element<'_, Self::Message, Self::Theme, Self::Renderer>
            {
                self.program.view(&self.state, window)
            }

            fn subscription(&self) -> Subscription<Self::Message> {
                self.program.subscription(&self.state)
            }

            fn theme(&self, window: window::Id) -> Self::Theme {
                self.program.theme(&self.state, window)
            }

            fn style(&self, theme: &Self::Theme) -> Appearance {
                self.program.style(&self.state, theme)
            }

            fn scale_factor(&self, window: window::Id) -> f64 {
                self.program.scale_factor(&self.state, window)
            }
        }

        #[allow(clippy::needless_update)]
        let renderer_settings = crate::graphics::Settings {
            default_font: settings.default_font,
            default_text_size: settings.default_text_size,
            antialiasing: if settings.antialiasing {
                Some(crate::graphics::Antialiasing::MSAAx4)
            } else {
                None
            },
            ..crate::graphics::Settings::default()
        };

        Ok(shell::program::run::<
            Instance<Self, I>,
            <Self::Renderer as compositor::Default>::Compositor,
        >(
            Settings {
                id: settings.id,
                fonts: settings.fonts,
                default_font: settings.default_font,
                default_text_size: settings.default_text_size,
                antialiasing: settings.antialiasing,
            }
            .into(),
            renderer_settings,
            window_settings,
            (self, initialize),
        )?)
    }
}

pub fn with_title<P: Program>(
    program: P,
    title: impl Fn(&P::State, window::Id) -> String,
) -> impl Program<State = P::State, Message = P::Message, Theme = P::Theme> {
    struct WithTitle<P, Title> {
        program: P,
        title: Title,
    }

    impl<P, Title> Program for WithTitle<P, Title>
    where
        P: Program,
        Title: Fn(&P::State, window::Id) -> String,
    {
        type State = P::State;
        type Message = P::Message;
        type Theme = P::Theme;
        type Renderer = P::Renderer;
        type Executor = P::Executor;

        fn title(&self, state: &Self::State, window: window::Id) -> String {
            (self.title)(state, window)
        }

        fn update(
            &self,
            state: &mut Self::State,
            message: Self::Message,
        ) -> Task<Self::Message> {
            self.program.update(state, message)
        }

        fn view<'a>(
            &self,
            state: &'a Self::State,
            window: window::Id,
        ) -> Element<'a, Self::Message, Self::Theme, Self::Renderer> {
            self.program.view(state, window)
        }

        fn theme(
            &self,
            state: &Self::State,
            window: window::Id,
        ) -> Self::Theme {
            self.program.theme(state, window)
        }

        fn subscription(
            &self,
            state: &Self::State,
        ) -> Subscription<Self::Message> {
            self.program.subscription(state)
        }

        fn style(
            &self,
            state: &Self::State,
            theme: &Self::Theme,
        ) -> Appearance {
            self.program.style(state, theme)
        }

        fn scale_factor(&self, state: &Self::State, window: window::Id) -> f64 {
            self.program.scale_factor(state, window)
        }
    }

    WithTitle { program, title }
}

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

        fn subscription(
            &self,
            state: &Self::State,
        ) -> Subscription<Self::Message> {
            (self.subscription)(state)
        }

        fn update(
            &self,
            state: &mut Self::State,
            message: Self::Message,
        ) -> Task<Self::Message> {
            self.program.update(state, message)
        }

        fn view<'a>(
            &self,
            state: &'a Self::State,
            window: window::Id,
        ) -> Element<'a, Self::Message, Self::Theme, Self::Renderer> {
            self.program.view(state, window)
        }

        fn title(&self, state: &Self::State, window: window::Id) -> String {
            self.program.title(state, window)
        }

        fn theme(
            &self,
            state: &Self::State,
            window: window::Id,
        ) -> Self::Theme {
            self.program.theme(state, window)
        }

        fn style(
            &self,
            state: &Self::State,
            theme: &Self::Theme,
        ) -> Appearance {
            self.program.style(state, theme)
        }

        fn scale_factor(&self, state: &Self::State, window: window::Id) -> f64 {
            self.program.scale_factor(state, window)
        }
    }

    WithSubscription {
        program,
        subscription: f,
    }
}

pub fn with_theme<P: Program>(
    program: P,
    f: impl Fn(&P::State, window::Id) -> P::Theme,
) -> impl Program<State = P::State, Message = P::Message, Theme = P::Theme> {
    struct WithTheme<P, F> {
        program: P,
        theme: F,
    }

    impl<P: Program, F> Program for WithTheme<P, F>
    where
        F: Fn(&P::State, window::Id) -> P::Theme,
    {
        type State = P::State;
        type Message = P::Message;
        type Theme = P::Theme;
        type Renderer = P::Renderer;
        type Executor = P::Executor;

        fn theme(
            &self,
            state: &Self::State,
            window: window::Id,
        ) -> Self::Theme {
            (self.theme)(state, window)
        }

        fn title(&self, state: &Self::State, window: window::Id) -> String {
            self.program.title(state, window)
        }

        fn update(
            &self,
            state: &mut Self::State,
            message: Self::Message,
        ) -> Task<Self::Message> {
            self.program.update(state, message)
        }

        fn view<'a>(
            &self,
            state: &'a Self::State,
            window: window::Id,
        ) -> Element<'a, Self::Message, Self::Theme, Self::Renderer> {
            self.program.view(state, window)
        }

        fn subscription(
            &self,
            state: &Self::State,
        ) -> Subscription<Self::Message> {
            self.program.subscription(state)
        }

        fn style(
            &self,
            state: &Self::State,
            theme: &Self::Theme,
        ) -> Appearance {
            self.program.style(state, theme)
        }

        fn scale_factor(&self, state: &Self::State, window: window::Id) -> f64 {
            self.program.scale_factor(state, window)
        }
    }

    WithTheme { program, theme: f }
}

pub fn with_style<P: Program>(
    program: P,
    f: impl Fn(&P::State, &P::Theme) -> Appearance,
) -> impl Program<State = P::State, Message = P::Message, Theme = P::Theme> {
    struct WithStyle<P, F> {
        program: P,
        style: F,
    }

    impl<P: Program, F> Program for WithStyle<P, F>
    where
        F: Fn(&P::State, &P::Theme) -> Appearance,
    {
        type State = P::State;
        type Message = P::Message;
        type Theme = P::Theme;
        type Renderer = P::Renderer;
        type Executor = P::Executor;

        fn style(
            &self,
            state: &Self::State,
            theme: &Self::Theme,
        ) -> Appearance {
            (self.style)(state, theme)
        }

        fn title(&self, state: &Self::State, window: window::Id) -> String {
            self.program.title(state, window)
        }

        fn update(
            &self,
            state: &mut Self::State,
            message: Self::Message,
        ) -> Task<Self::Message> {
            self.program.update(state, message)
        }

        fn view<'a>(
            &self,
            state: &'a Self::State,
            window: window::Id,
        ) -> Element<'a, Self::Message, Self::Theme, Self::Renderer> {
            self.program.view(state, window)
        }

        fn subscription(
            &self,
            state: &Self::State,
        ) -> Subscription<Self::Message> {
            self.program.subscription(state)
        }

        fn theme(
            &self,
            state: &Self::State,
            window: window::Id,
        ) -> Self::Theme {
            self.program.theme(state, window)
        }

        fn scale_factor(&self, state: &Self::State, window: window::Id) -> f64 {
            self.program.scale_factor(state, window)
        }
    }

    WithStyle { program, style: f }
}

pub fn with_scale_factor<P: Program>(
    program: P,
    f: impl Fn(&P::State, window::Id) -> f64,
) -> impl Program<State = P::State, Message = P::Message, Theme = P::Theme> {
    struct WithScaleFactor<P, F> {
        program: P,
        scale_factor: F,
    }

    impl<P: Program, F> Program for WithScaleFactor<P, F>
    where
        F: Fn(&P::State, window::Id) -> f64,
    {
        type State = P::State;
        type Message = P::Message;
        type Theme = P::Theme;
        type Renderer = P::Renderer;
        type Executor = P::Executor;

        fn title(&self, state: &Self::State, window: window::Id) -> String {
            self.program.title(state, window)
        }

        fn update(
            &self,
            state: &mut Self::State,
            message: Self::Message,
        ) -> Task<Self::Message> {
            self.program.update(state, message)
        }

        fn view<'a>(
            &self,
            state: &'a Self::State,
            window: window::Id,
        ) -> Element<'a, Self::Message, Self::Theme, Self::Renderer> {
            self.program.view(state, window)
        }

        fn subscription(
            &self,
            state: &Self::State,
        ) -> Subscription<Self::Message> {
            self.program.subscription(state)
        }

        fn theme(
            &self,
            state: &Self::State,
            window: window::Id,
        ) -> Self::Theme {
            self.program.theme(state, window)
        }

        fn style(
            &self,
            state: &Self::State,
            theme: &Self::Theme,
        ) -> Appearance {
            self.program.style(state, theme)
        }

        fn scale_factor(&self, state: &Self::State, window: window::Id) -> f64 {
            (self.scale_factor)(state, window)
        }
    }

    WithScaleFactor {
        program,
        scale_factor: f,
    }
}

/// The renderer of some [`Program`].
pub trait Renderer: text::Renderer + compositor::Default {}

impl<T> Renderer for T where T: text::Renderer + compositor::Default {}
