use iced::event::{self, Event};
use iced::executor;
use iced::keyboard;
use iced::theme;
use iced::widget::{
    self, button, column, container, horizontal_space, pick_list, row, text,
    text_input,
};
use iced::{
    Alignment, Application, Command, Element, Length, Settings, Subscription,
};

use modal::Modal;
use std::fmt;

pub fn main() -> iced::Result {
    App::run(Settings::default())
}

#[derive(Default)]
struct App {
    show_modal: bool,
    email: String,
    password: String,
    plan: Plan,
}

#[derive(Debug, Clone)]
enum Message {
    ShowModal,
    HideModal,
    Email(String),
    Password(String),
    Plan(Plan),
    Submit,
    Event(Event),
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (App::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Modal - Iced")
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        event::listen().map(Message::Event)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ShowModal => {
                self.show_modal = true;
                widget::focus_next()
            }
            Message::HideModal => {
                self.hide_modal();
                Command::none()
            }
            Message::Email(email) => {
                self.email = email;
                Command::none()
            }
            Message::Password(password) => {
                self.password = password;
                Command::none()
            }
            Message::Plan(plan) => {
                self.plan = plan;
                Command::none()
            }
            Message::Submit => {
                if !self.email.is_empty() && !self.password.is_empty() {
                    self.hide_modal();
                }

                Command::none()
            }
            Message::Event(event) => match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key_code: keyboard::KeyCode::Tab,
                    modifiers,
                }) => {
                    if modifiers.shift() {
                        widget::focus_previous()
                    } else {
                        widget::focus_next()
                    }
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key_code: keyboard::KeyCode::Escape,
                    ..
                }) => {
                    self.hide_modal();
                    Command::none()
                }
                _ => Command::none(),
            },
        }
    }

    fn view(&self) -> Element<Message> {
        let content = container(
            column![
                row![
                    text("Top Left"),
                    horizontal_space(Length::Fill),
                    text("Top Right")
                ]
                .align_items(Alignment::Start)
                .height(Length::Fill),
                container(
                    button(text("Show Modal")).on_press(Message::ShowModal)
                )
                .center_x()
                .center_y()
                .width(Length::Fill)
                .height(Length::Fill),
                row![
                    text("Bottom Left"),
                    horizontal_space(Length::Fill),
                    text("Bottom Right")
                ]
                .align_items(Alignment::End)
                .height(Length::Fill),
            ]
            .height(Length::Fill),
        )
        .padding(10)
        .width(Length::Fill)
        .height(Length::Fill);

        if self.show_modal {
            let modal = container(
                column![
                    text("Sign Up").size(24),
                    column![
                        column![
                            text("Email").size(12),
                            text_input("abc@123.com", &self.email,)
                                .on_input(Message::Email)
                                .on_submit(Message::Submit)
                                .padding(5),
                        ]
                        .spacing(5),
                        column![
                            text("Password").size(12),
                            text_input("", &self.password)
                                .on_input(Message::Password)
                                .on_submit(Message::Submit)
                                .password()
                                .padding(5),
                        ]
                        .spacing(5),
                        column![
                            text("Plan").size(12),
                            pick_list(
                                Plan::ALL,
                                Some(self.plan),
                                Message::Plan
                            )
                            .padding(5),
                        ]
                        .spacing(5),
                        button(text("Submit")).on_press(Message::HideModal),
                    ]
                    .spacing(10)
                ]
                .spacing(20),
            )
            .width(300)
            .padding(10)
            .style(theme::Container::Box);

            Modal::new(content, modal)
                .on_blur(Message::HideModal)
                .into()
        } else {
            content.into()
        }
    }
}

impl App {
    fn hide_modal(&mut self) {
        self.show_modal = false;
        self.email.clear();
        self.password.clear();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Plan {
    #[default]
    Basic,
    Pro,
    Enterprise,
}

impl Plan {
    pub const ALL: &'static [Self] =
        &[Self::Basic, Self::Pro, Self::Enterprise];
}

impl fmt::Display for Plan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Plan::Basic => "Basic",
            Plan::Pro => "Pro",
            Plan::Enterprise => "Enterprise",
        }
        .fmt(f)
    }
}

mod modal {
    use iced::advanced::layout::{self, Layout};
    use iced::advanced::overlay;
    use iced::advanced::renderer;
    use iced::advanced::widget::{self, Widget};
    use iced::advanced::{self, Clipboard, Shell};
    use iced::alignment::Alignment;
    use iced::event;
    use iced::mouse;
    use iced::{
        BorderRadius, Color, Element, Event, Length, Point, Rectangle, Size,
        Vector,
    };

    /// A widget that centers a modal element over some base element
    pub struct Modal<'a, Message, Renderer> {
        base: Element<'a, Message, Renderer>,
        modal: Element<'a, Message, Renderer>,
        on_blur: Option<Message>,
    }

    impl<'a, Message, Renderer> Modal<'a, Message, Renderer> {
        /// Returns a new [`Modal`]
        pub fn new(
            base: impl Into<Element<'a, Message, Renderer>>,
            modal: impl Into<Element<'a, Message, Renderer>>,
        ) -> Self {
            Self {
                base: base.into(),
                modal: modal.into(),
                on_blur: None,
            }
        }

        /// Sets the message that will be produces when the background
        /// of the [`Modal`] is pressed
        pub fn on_blur(self, on_blur: Message) -> Self {
            Self {
                on_blur: Some(on_blur),
                ..self
            }
        }
    }

    impl<'a, Message, Renderer> Widget<Message, Renderer>
        for Modal<'a, Message, Renderer>
    where
        Renderer: advanced::Renderer,
        Message: Clone,
    {
        fn children(&self) -> Vec<widget::Tree> {
            vec![
                widget::Tree::new(&self.base),
                widget::Tree::new(&self.modal),
            ]
        }

        fn diff(&self, tree: &mut widget::Tree) {
            tree.diff_children(&[&self.base, &self.modal]);
        }

        fn size(&self) -> Size<Length> {
            self.base.as_widget().size()
        }

        fn layout(
            &self,
            tree: &mut widget::Tree,
            renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            self.base.as_widget().layout(
                &mut tree.children[0],
                renderer,
                limits,
            )
        }

        fn on_event(
            &mut self,
            state: &mut widget::Tree,
            event: Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
            viewport: &Rectangle,
        ) -> event::Status {
            self.base.as_widget_mut().on_event(
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
            state: &widget::Tree,
            renderer: &mut Renderer,
            theme: &<Renderer as advanced::Renderer>::Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            self.base.as_widget().draw(
                &state.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
        }

        fn overlay<'b>(
            &'b mut self,
            state: &'b mut widget::Tree,
            layout: Layout<'_>,
            _renderer: &Renderer,
        ) -> Option<overlay::Element<'b, Message, Renderer>> {
            Some(overlay::Element::new(
                layout.position(),
                Box::new(Overlay {
                    content: &mut self.modal,
                    tree: &mut state.children[1],
                    size: layout.bounds().size(),
                    on_blur: self.on_blur.clone(),
                }),
            ))
        }

        fn mouse_interaction(
            &self,
            state: &widget::Tree,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction {
            self.base.as_widget().mouse_interaction(
                &state.children[0],
                layout,
                cursor,
                viewport,
                renderer,
            )
        }

        fn operate(
            &self,
            state: &mut widget::Tree,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn widget::Operation<Message>,
        ) {
            self.base.as_widget().operate(
                &mut state.children[0],
                layout,
                renderer,
                operation,
            );
        }
    }

    struct Overlay<'a, 'b, Message, Renderer> {
        content: &'b mut Element<'a, Message, Renderer>,
        tree: &'b mut widget::Tree,
        size: Size,
        on_blur: Option<Message>,
    }

    impl<'a, 'b, Message, Renderer> overlay::Overlay<Message, Renderer>
        for Overlay<'a, 'b, Message, Renderer>
    where
        Renderer: advanced::Renderer,
        Message: Clone,
    {
        fn layout(
            &mut self,
            renderer: &Renderer,
            _bounds: Size,
            position: Point,
            _translation: Vector,
        ) -> layout::Node {
            let limits = layout::Limits::new(Size::ZERO, self.size)
                .width(Length::Fill)
                .height(Length::Fill);

            let child = self
                .content
                .as_widget()
                .layout(self.tree, renderer, &limits)
                .align(Alignment::Center, Alignment::Center, limits.max());

            layout::Node::with_children(self.size, vec![child])
                .move_to(position)
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
            let content_bounds = layout.children().next().unwrap().bounds();

            if let Some(message) = self.on_blur.as_ref() {
                if let Event::Mouse(mouse::Event::ButtonPressed(
                    mouse::Button::Left,
                )) = &event
                {
                    if !cursor.is_over(content_bounds) {
                        shell.publish(message.clone());
                        return event::Status::Captured;
                    }
                }
            }

            self.content.as_widget_mut().on_event(
                self.tree,
                event,
                layout.children().next().unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                &layout.bounds(),
            )
        }

        fn draw(
            &self,
            renderer: &mut Renderer,
            theme: &Renderer::Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
        ) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border_radius: BorderRadius::default(),
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
                Color {
                    a: 0.80,
                    ..Color::BLACK
                },
            );

            self.content.as_widget().draw(
                self.tree,
                renderer,
                theme,
                style,
                layout.children().next().unwrap(),
                cursor,
                &layout.bounds(),
            );
        }

        fn operate(
            &mut self,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn widget::Operation<Message>,
        ) {
            self.content.as_widget().operate(
                self.tree,
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        }

        fn mouse_interaction(
            &self,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction {
            self.content.as_widget().mouse_interaction(
                self.tree,
                layout.children().next().unwrap(),
                cursor,
                viewport,
                renderer,
            )
        }

        fn overlay<'c>(
            &'c mut self,
            layout: Layout<'_>,
            renderer: &Renderer,
        ) -> Option<overlay::Element<'c, Message, Renderer>> {
            self.content.as_widget_mut().overlay(
                self.tree,
                layout.children().next().unwrap(),
                renderer,
            )
        }
    }

    impl<'a, Message, Renderer> From<Modal<'a, Message, Renderer>>
        for Element<'a, Message, Renderer>
    where
        Renderer: 'a + advanced::Renderer,
        Message: 'a + Clone,
    {
        fn from(modal: Modal<'a, Message, Renderer>) -> Self {
            Element::new(modal)
        }
    }
}
