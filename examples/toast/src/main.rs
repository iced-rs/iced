use iced::event::{self, Event};
use iced::keyboard;
use iced::keyboard::key;
use iced::widget::{
    self, button, center, column, pick_list, row, slider, text, text_input,
};
use iced::{Center, Element, Fill, Subscription, Task};

use toast::{Status, Toast};

pub fn main() -> iced::Result {
    iced::application("Toast - Iced", App::update, App::view)
        .subscription(App::subscription)
        .run()
}

struct App {
    toasts: Vec<Toast>,
    editing: Toast,
    timeout_secs: u64,
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
enum Message {
    Add,
    Close(usize),
    Title(String),
    Body(String),
    Status(Status),
    Timeout(f64),
    Event(Event),
}

impl App {
    fn new() -> Self {
        App {
            toasts: vec![Toast {
                title: "Example Toast".into(),
                body: "Add more toasts in the form below!".into(),
                status: Status::Primary,
            }],
            timeout_secs: toast::DEFAULT_TIMEOUT,
            editing: Toast::default(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::Event)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Add => {
                if !self.editing.title.is_empty()
                    && !self.editing.body.is_empty()
                {
                    self.toasts.push(std::mem::take(&mut self.editing));
                }
                Task::none()
            }
            Message::Close(index) => {
                self.toasts.remove(index);
                Task::none()
            }
            Message::Title(title) => {
                self.editing.title = title;
                Task::none()
            }
            Message::Body(body) => {
                self.editing.body = body;
                Task::none()
            }
            Message::Status(status) => {
                self.editing.status = status;
                Task::none()
            }
            Message::Timeout(timeout) => {
                self.timeout_secs = timeout as u64;
                Task::none()
            }
            Message::Event(Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Tab),
                modifiers,
                ..
            })) if modifiers.shift() => widget::focus_previous(),
            Message::Event(Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Tab),
                ..
            })) => widget::focus_next(),
            Message::Event(_) => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let subtitle = |title, content: Element<'static, Message>| {
            column![text(title).size(14), content].spacing(5)
        };

        let add_toast = button("Add Toast").on_press_maybe(
            (!self.editing.body.is_empty() && !self.editing.title.is_empty())
                .then_some(Message::Add),
        );

        let content = center(
            column![
                subtitle(
                    "Title",
                    text_input("", &self.editing.title)
                        .on_input(Message::Title)
                        .on_submit(Message::Add)
                        .into()
                ),
                subtitle(
                    "Message",
                    text_input("", &self.editing.body)
                        .on_input(Message::Body)
                        .on_submit(Message::Add)
                        .into()
                ),
                subtitle(
                    "Status",
                    pick_list(
                        toast::Status::ALL,
                        Some(self.editing.status),
                        Message::Status
                    )
                    .width(Fill)
                    .into()
                ),
                subtitle(
                    "Timeout",
                    row![
                        text!("{:0>2} sec", self.timeout_secs),
                        slider(
                            1.0..=30.0,
                            self.timeout_secs as f64,
                            Message::Timeout
                        )
                        .step(1.0)
                    ]
                    .spacing(5)
                    .into()
                ),
                column![add_toast].align_x(Center)
            ]
            .spacing(10)
            .max_width(200),
        );

        toast::Manager::new(content, &self.toasts, Message::Close)
            .timeout(self.timeout_secs)
            .into()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

mod toast {
    use std::fmt;
    use std::time::{Duration, Instant};

    use iced::advanced::layout::{self, Layout};
    use iced::advanced::overlay;
    use iced::advanced::renderer;
    use iced::advanced::widget::{self, Operation, Tree};
    use iced::advanced::{Clipboard, Shell, Widget};
    use iced::event::{self, Event};
    use iced::mouse;
    use iced::theme;
    use iced::widget::{
        button, column, container, horizontal_rule, horizontal_space, row, text,
    };
    use iced::window;
    use iced::{
        Alignment, Center, Element, Fill, Length, Point, Rectangle, Renderer,
        Size, Theme, Vector,
    };

    pub const DEFAULT_TIMEOUT: u64 = 5;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum Status {
        #[default]
        Primary,
        Secondary,
        Success,
        Danger,
    }

    impl Status {
        pub const ALL: &'static [Self] =
            &[Self::Primary, Self::Secondary, Self::Success, Self::Danger];
    }

    impl fmt::Display for Status {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Status::Primary => "Primary",
                Status::Secondary => "Secondary",
                Status::Success => "Success",
                Status::Danger => "Danger",
            }
            .fmt(f)
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct Toast {
        pub title: String,
        pub body: String,
        pub status: Status,
    }

    pub struct Manager<'a, Message> {
        content: Element<'a, Message>,
        toasts: Vec<Element<'a, Message>>,
        timeout_secs: u64,
        on_close: Box<dyn Fn(usize) -> Message + 'a>,
    }

    impl<'a, Message> Manager<'a, Message>
    where
        Message: 'a + Clone,
    {
        pub fn new(
            content: impl Into<Element<'a, Message>>,
            toasts: &'a [Toast],
            on_close: impl Fn(usize) -> Message + 'a,
        ) -> Self {
            let toasts = toasts
                .iter()
                .enumerate()
                .map(|(index, toast)| {
                    container(column![
                        container(
                            row![
                                text(toast.title.as_str()),
                                horizontal_space(),
                                button("X")
                                    .on_press((on_close)(index))
                                    .padding(3),
                            ]
                            .align_y(Center)
                        )
                        .width(Fill)
                        .padding(5)
                        .style(match toast.status {
                            Status::Primary => primary,
                            Status::Secondary => secondary,
                            Status::Success => success,
                            Status::Danger => danger,
                        }),
                        horizontal_rule(1),
                        container(text(toast.body.as_str()))
                            .width(Fill)
                            .padding(5)
                            .style(container::rounded_box),
                    ])
                    .max_width(200)
                    .into()
                })
                .collect();

            Self {
                content: content.into(),
                toasts,
                timeout_secs: DEFAULT_TIMEOUT,
                on_close: Box::new(on_close),
            }
        }

        pub fn timeout(self, seconds: u64) -> Self {
            Self {
                timeout_secs: seconds,
                ..self
            }
        }
    }

    impl<'a, Message> Widget<Message, Theme, Renderer> for Manager<'a, Message> {
        fn size(&self) -> Size<Length> {
            self.content.as_widget().size()
        }

        fn layout(
            &self,
            tree: &mut Tree,
            renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            self.content.as_widget().layout(
                &mut tree.children[0],
                renderer,
                limits,
            )
        }

        fn tag(&self) -> widget::tree::Tag {
            struct Marker;
            widget::tree::Tag::of::<Marker>()
        }

        fn state(&self) -> widget::tree::State {
            widget::tree::State::new(Vec::<Option<Instant>>::new())
        }

        fn children(&self) -> Vec<Tree> {
            std::iter::once(Tree::new(&self.content))
                .chain(self.toasts.iter().map(Tree::new))
                .collect()
        }

        fn diff(&self, tree: &mut Tree) {
            let instants = tree.state.downcast_mut::<Vec<Option<Instant>>>();

            // Invalidating removed instants to None allows us to remove
            // them here so that diffing for removed / new toast instants
            // is accurate
            instants.retain(Option::is_some);

            match (instants.len(), self.toasts.len()) {
                (old, new) if old > new => {
                    instants.truncate(new);
                }
                (old, new) if old < new => {
                    instants.extend(
                        std::iter::repeat(Some(Instant::now())).take(new - old),
                    );
                }
                _ => {}
            }

            tree.diff_children(
                &std::iter::once(&self.content)
                    .chain(self.toasts.iter())
                    .collect::<Vec<_>>(),
            );
        }

        fn operate(
            &self,
            state: &mut Tree,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn Operation,
        ) {
            operation.container(None, layout.bounds(), &mut |operation| {
                self.content.as_widget().operate(
                    &mut state.children[0],
                    layout,
                    renderer,
                    operation,
                );
            });
        }

        fn on_event(
            &mut self,
            state: &mut Tree,
            event: Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
            viewport: &Rectangle,
        ) -> event::Status {
            self.content.as_widget_mut().on_event(
                &mut state.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            )
        }

        fn draw(
            &self,
            state: &Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            self.content.as_widget().draw(
                &state.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
        }

        fn mouse_interaction(
            &self,
            state: &Tree,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction {
            self.content.as_widget().mouse_interaction(
                &state.children[0],
                layout,
                cursor,
                viewport,
                renderer,
            )
        }

        fn overlay<'b>(
            &'b mut self,
            state: &'b mut Tree,
            layout: Layout<'_>,
            renderer: &Renderer,
            translation: Vector,
        ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
            let instants = state.state.downcast_mut::<Vec<Option<Instant>>>();

            let (content_state, toasts_state) = state.children.split_at_mut(1);

            let content = self.content.as_widget_mut().overlay(
                &mut content_state[0],
                layout,
                renderer,
                translation,
            );

            let toasts = (!self.toasts.is_empty()).then(|| {
                overlay::Element::new(Box::new(Overlay {
                    position: layout.bounds().position() + translation,
                    toasts: &mut self.toasts,
                    state: toasts_state,
                    instants,
                    on_close: &self.on_close,
                    timeout_secs: self.timeout_secs,
                }))
            });
            let overlays =
                content.into_iter().chain(toasts).collect::<Vec<_>>();

            (!overlays.is_empty())
                .then(|| overlay::Group::with_children(overlays).overlay())
        }
    }

    struct Overlay<'a, 'b, Message> {
        position: Point,
        toasts: &'b mut [Element<'a, Message>],
        state: &'b mut [Tree],
        instants: &'b mut [Option<Instant>],
        on_close: &'b dyn Fn(usize) -> Message,
        timeout_secs: u64,
    }

    impl<'a, 'b, Message> overlay::Overlay<Message, Theme, Renderer>
        for Overlay<'a, 'b, Message>
    {
        fn layout(
            &mut self,
            renderer: &Renderer,
            bounds: Size,
        ) -> layout::Node {
            let limits = layout::Limits::new(Size::ZERO, bounds);

            layout::flex::resolve(
                layout::flex::Axis::Vertical,
                renderer,
                &limits,
                Fill,
                Fill,
                10.into(),
                10.0,
                Alignment::End,
                self.toasts,
                self.state,
            )
            .translate(Vector::new(self.position.x, self.position.y))
        }

        fn on_event(
            &mut self,
            event: Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
        ) -> event::Status {
            if let Event::Window(window::Event::RedrawRequested(now)) = &event {
                let mut next_redraw: Option<window::RedrawRequest> = None;

                self.instants.iter_mut().enumerate().for_each(
                    |(index, maybe_instant)| {
                        if let Some(instant) = maybe_instant.as_mut() {
                            let remaining =
                                Duration::from_secs(self.timeout_secs)
                                    .saturating_sub(instant.elapsed());

                            if remaining == Duration::ZERO {
                                maybe_instant.take();
                                shell.publish((self.on_close)(index));
                                next_redraw =
                                    Some(window::RedrawRequest::NextFrame);
                            } else {
                                let redraw_at =
                                    window::RedrawRequest::At(*now + remaining);
                                next_redraw = next_redraw
                                    .map(|redraw| redraw.min(redraw_at))
                                    .or(Some(redraw_at));
                            }
                        }
                    },
                );

                if let Some(redraw) = next_redraw {
                    shell.request_redraw(redraw);
                }
            }

            let viewport = layout.bounds();

            self.toasts
                .iter_mut()
                .zip(self.state.iter_mut())
                .zip(layout.children())
                .zip(self.instants.iter_mut())
                .map(|(((child, state), layout), instant)| {
                    let mut local_messages = vec![];
                    let mut local_shell = Shell::new(&mut local_messages);

                    let status = child.as_widget_mut().on_event(
                        state,
                        event.clone(),
                        layout,
                        cursor,
                        renderer,
                        clipboard,
                        &mut local_shell,
                        &viewport,
                    );

                    if !local_shell.is_empty() {
                        instant.take();
                    }

                    shell.merge(local_shell, std::convert::identity);

                    status
                })
                .fold(event::Status::Ignored, event::Status::merge)
        }

        fn draw(
            &self,
            renderer: &mut Renderer,
            theme: &Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
        ) {
            let viewport = layout.bounds();

            for ((child, state), layout) in self
                .toasts
                .iter()
                .zip(self.state.iter())
                .zip(layout.children())
            {
                child.as_widget().draw(
                    state, renderer, theme, style, layout, cursor, &viewport,
                );
            }
        }

        fn operate(
            &mut self,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn widget::Operation,
        ) {
            operation.container(None, layout.bounds(), &mut |operation| {
                self.toasts
                    .iter()
                    .zip(self.state.iter_mut())
                    .zip(layout.children())
                    .for_each(|((child, state), layout)| {
                        child
                            .as_widget()
                            .operate(state, layout, renderer, operation);
                    });
            });
        }

        fn mouse_interaction(
            &self,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction {
            self.toasts
                .iter()
                .zip(self.state.iter())
                .zip(layout.children())
                .map(|((child, state), layout)| {
                    child.as_widget().mouse_interaction(
                        state, layout, cursor, viewport, renderer,
                    )
                })
                .max()
                .unwrap_or_default()
        }

        fn is_over(
            &self,
            layout: Layout<'_>,
            _renderer: &Renderer,
            cursor_position: Point,
        ) -> bool {
            layout
                .children()
                .any(|layout| layout.bounds().contains(cursor_position))
        }
    }

    impl<'a, Message> From<Manager<'a, Message>> for Element<'a, Message>
    where
        Message: 'a,
    {
        fn from(manager: Manager<'a, Message>) -> Self {
            Element::new(manager)
        }
    }

    fn styled(pair: theme::palette::Pair) -> container::Style {
        container::Style {
            background: Some(pair.color.into()),
            text_color: pair.text.into(),
            ..Default::default()
        }
    }

    fn primary(theme: &Theme) -> container::Style {
        let palette = theme.extended_palette();

        styled(palette.primary.weak)
    }

    fn secondary(theme: &Theme) -> container::Style {
        let palette = theme.extended_palette();

        styled(palette.secondary.weak)
    }

    fn success(theme: &Theme) -> container::Style {
        let palette = theme.extended_palette();

        styled(palette.success.weak)
    }

    fn danger(theme: &Theme) -> container::Style {
        let palette = theme.extended_palette();

        styled(palette.danger.weak)
    }
}
