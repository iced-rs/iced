use iced::widget::{
    self, button, column, container, pick_list, row, slider, text, text_input,
};
use iced::{
    executor, keyboard, subscription, Alignment, Application, Command, Element,
    Event, Length, Settings, Subscription,
};

use toast::{Status, Toast};

pub fn main() -> iced::Result {
    App::run(Settings::default())
}

#[derive(Default)]
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

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            App {
                toasts: vec![Toast {
                    title: "Example Toast".into(),
                    body: "Add more toasts in the form below!".into(),
                    status: Status::Primary,
                }],
                timeout_secs: toast::DEFAULT_TIMEOUT,
                ..Default::default()
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Toast - Iced")
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        subscription::events().map(Message::Event)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Add => {
                if !self.editing.title.is_empty()
                    && !self.editing.body.is_empty()
                {
                    self.toasts.push(std::mem::take(&mut self.editing));
                }
                Command::none()
            }
            Message::Close(index) => {
                self.toasts.remove(index);
                Command::none()
            }
            Message::Title(title) => {
                self.editing.title = title;
                Command::none()
            }
            Message::Body(body) => {
                self.editing.body = body;
                Command::none()
            }
            Message::Status(status) => {
                self.editing.status = status;
                Command::none()
            }
            Message::Timeout(timeout) => {
                self.timeout_secs = timeout as u64;
                Command::none()
            }
            Message::Event(Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: keyboard::KeyCode::Tab,
                modifiers,
            })) if modifiers.shift() => widget::focus_previous(),
            Message::Event(Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: keyboard::KeyCode::Tab,
                ..
            })) => widget::focus_next(),
            Message::Event(_) => Command::none(),
        }
    }

    fn view<'a>(&'a self) -> Element<'a, Message> {
        let subtitle = |title, content: Element<'a, Message>| {
            column![text(title).size(14), content]
                .width(Length::Fill)
                .spacing(5)
        };

        let mut add_toast = button("Add Toast");

        if !self.editing.body.is_empty() && !self.editing.title.is_empty() {
            add_toast = add_toast.on_press(Message::Add);
        }

        let content = container(
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
                    .width(Length::Fill)
                    .into()
                ),
                subtitle(
                    "Timeout",
                    row![
                        text(format!("{:0>2} sec", self.timeout_secs)),
                        slider(
                            1.0..=30.0,
                            self.timeout_secs as f64,
                            Message::Timeout
                        )
                        .step(1.0)
                        .width(Length::Fill)
                    ]
                    .spacing(5)
                    .into()
                ),
                column![add_toast]
                    .width(Length::Fill)
                    .align_items(Alignment::End)
            ]
            .spacing(10)
            .max_width(200),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y();

        toast::Manager::new(content, &self.toasts, Message::Close)
            .timeout(self.timeout_secs)
            .into()
    }
}

mod toast {
    use std::fmt;
    use std::time::{Duration, Instant};

    use iced::advanced;
    use iced::advanced::layout::{self, Layout};
    use iced::advanced::overlay;
    use iced::advanced::renderer;
    use iced::advanced::widget::{
        self, Operation, OperationOutputWrapper, Tree,
    };
    use iced::advanced::{Clipboard, Shell, Widget};
    use iced::event::{self, Event};
    use iced::mouse;
    use iced::theme;
    use iced::widget::{
        button, column, container, horizontal_rule, horizontal_space, row, text,
    };
    use iced::window;
    use iced::{
        Alignment, Element, Length, Point, Rectangle, Renderer, Size, Theme,
        Vector,
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
        pub const ALL: &[Self] =
            &[Self::Primary, Self::Secondary, Self::Success, Self::Danger];
    }

    impl container::StyleSheet for Status {
        type Style = Theme;

        fn appearance(&self, theme: &Theme) -> container::Appearance {
            let palette = theme.extended_palette();

            let pair = match self {
                Status::Primary => palette.primary.weak,
                Status::Secondary => palette.secondary.weak,
                Status::Success => palette.success.weak,
                Status::Danger => palette.danger.weak,
            };

            container::Appearance {
                background: pair.color.into(),
                text_color: pair.text.into(),
                ..Default::default()
            }
        }
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
                                horizontal_space(Length::Fill),
                                button("X")
                                    .on_press((on_close)(index))
                                    .padding(3),
                            ]
                            .align_items(Alignment::Center)
                        )
                        .width(Length::Fill)
                        .padding(5)
                        .style(
                            theme::Container::Custom(Box::new(toast.status))
                        ),
                        horizontal_rule(1),
                        container(text(toast.body.as_str()))
                            .width(Length::Fill)
                            .padding(5)
                            .style(theme::Container::Box),
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

    impl<'a, Message> Widget<Message, Renderer> for Manager<'a, Message> {
        fn width(&self) -> Length {
            self.content.as_widget().width()
        }

        fn height(&self) -> Length {
            self.content.as_widget().height()
        }

        fn layout(
            &self,
            renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            self.content.as_widget().layout(renderer, limits)
        }

        fn tag(&self) -> widget::tree::Tag {
            struct Marker(Vec<Instant>);
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

        fn diff(&mut self, tree: &mut Tree) {
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
                &mut std::iter::once(&mut self.content)
                    .chain(self.toasts.iter_mut())
                    .collect::<Vec<_>>(),
            );
        }

        fn operate(
            &self,
            state: &mut Tree,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn Operation<OperationOutputWrapper<Message>>,
        ) {
            operation.container(None, &mut |operation| {
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
            cursor_position: Point,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
        ) -> event::Status {
            self.content.as_widget_mut().on_event(
                &mut state.children[0],
                event,
                layout,
                cursor_position,
                renderer,
                clipboard,
                shell,
            )
        }

        fn draw(
            &self,
            state: &Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor_position: Point,
            viewport: &Rectangle,
        ) {
            self.content.as_widget().draw(
                &state.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor_position,
                viewport,
            );
        }

        fn mouse_interaction(
            &self,
            state: &Tree,
            layout: Layout<'_>,
            cursor_position: Point,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction {
            self.content.as_widget().mouse_interaction(
                &state.children[0],
                layout,
                cursor_position,
                viewport,
                renderer,
            )
        }

        fn overlay<'b>(
            &'b mut self,
            state: &'b mut Tree,
            layout: Layout<'_>,
            renderer: &Renderer,
        ) -> Option<overlay::Element<'b, Message, Renderer>> {
            let instants = state.state.downcast_mut::<Vec<Option<Instant>>>();

            let (content_state, toasts_state) = state.children.split_at_mut(1);

            let content = self.content.as_widget_mut().overlay(
                &mut content_state[0],
                layout,
                renderer,
            );

            let toasts = (!self.toasts.is_empty()).then(|| {
                overlay::Element::new(
                    layout.bounds().position(),
                    Box::new(Overlay {
                        toasts: &mut self.toasts,
                        state: toasts_state,
                        instants,
                        on_close: &self.on_close,
                        timeout_secs: self.timeout_secs,
                    }),
                )
            });
            let overlays =
                content.into_iter().chain(toasts).collect::<Vec<_>>();

            (!overlays.is_empty())
                .then(|| overlay::Group::with_children(overlays).overlay())
        }
    }

    struct Overlay<'a, 'b, Message> {
        toasts: &'b mut [Element<'a, Message>],
        state: &'b mut [Tree],
        instants: &'b mut [Option<Instant>],
        on_close: &'b dyn Fn(usize) -> Message,
        timeout_secs: u64,
    }

    impl<'a, 'b, Message> overlay::Overlay<Message, Renderer>
        for Overlay<'a, 'b, Message>
    {
        fn layout(
            &self,
            renderer: &Renderer,
            bounds: Size,
            position: Point,
        ) -> layout::Node {
            let limits = layout::Limits::new(Size::ZERO, bounds)
                .width(Length::Fill)
                .height(Length::Fill);

            layout::flex::resolve(
                layout::flex::Axis::Vertical,
                renderer,
                &limits,
                10.into(),
                10.0,
                Alignment::End,
                self.toasts,
            )
            .translate(Vector::new(position.x, position.y))
        }

        fn on_event(
            &mut self,
            event: Event,
            layout: Layout<'_>,
            cursor_position: Point,
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
                        cursor_position,
                        renderer,
                        clipboard,
                        &mut local_shell,
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
            theme: &<Renderer as advanced::Renderer>::Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor_position: Point,
        ) {
            let viewport = layout.bounds();

            for ((child, state), layout) in self
                .toasts
                .iter()
                .zip(self.state.iter())
                .zip(layout.children())
            {
                child.as_widget().draw(
                    state,
                    renderer,
                    theme,
                    style,
                    layout,
                    cursor_position,
                    &viewport,
                );
            }
        }

        fn operate(
            &mut self,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn Operation<OperationOutputWrapper<Message>>,
        ) {
            operation.container(None, &mut |operation| {
                self.toasts
                    .iter()
                    .zip(self.state.iter_mut())
                    .zip(layout.children())
                    .for_each(|((child, state), layout)| {
                        child
                            .as_widget()
                            .operate(state, layout, renderer, operation);
                    })
            });
        }

        fn mouse_interaction(
            &self,
            layout: Layout<'_>,
            cursor_position: Point,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction {
            self.toasts
                .iter()
                .zip(self.state.iter())
                .zip(layout.children())
                .map(|((child, state), layout)| {
                    child.as_widget().mouse_interaction(
                        state,
                        layout,
                        cursor_position,
                        viewport,
                        renderer,
                    )
                })
                .max()
                .unwrap_or_default()
        }

        fn is_over(&self, layout: Layout<'_>, cursor_position: Point) -> bool {
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
}
