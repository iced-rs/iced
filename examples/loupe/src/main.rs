use iced::widget::{button, center, column, text};
use iced::{Center, Element};

use loupe::loupe;

pub fn main() -> iced::Result {
    iced::run("Loupe - Iced", Loupe::update, Loupe::view)
}

#[derive(Default)]
struct Loupe {
    value: i64,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Increment,
    Decrement,
}

impl Loupe {
    fn update(&mut self, message: Message) {
        match message {
            Message::Increment => {
                self.value += 1;
            }
            Message::Decrement => {
                self.value -= 1;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        center(loupe(
            3.0,
            column![
                button("Increment").on_press(Message::Increment),
                text(self.value).size(50),
                button("Decrement").on_press(Message::Decrement)
            ]
            .padding(20)
            .align_x(Center),
        ))
        .into()
    }
}

mod loupe {
    use iced::advanced::layout::{self, Layout};
    use iced::advanced::renderer;
    use iced::advanced::widget::{self, Widget};
    use iced::advanced::Renderer as _;
    use iced::mouse;
    use iced::{
        Color, Element, Length, Rectangle, Renderer, Size, Theme,
        Transformation,
    };

    pub fn loupe<'a, Message>(
        zoom: f32,
        content: impl Into<Element<'a, Message>>,
    ) -> Loupe<'a, Message>
    where
        Message: 'static,
    {
        Loupe {
            zoom,
            content: content.into().explain(Color::BLACK),
        }
    }

    pub struct Loupe<'a, Message> {
        zoom: f32,
        content: Element<'a, Message>,
    }

    impl<'a, Message> Widget<Message, Theme, Renderer> for Loupe<'a, Message> {
        fn tag(&self) -> widget::tree::Tag {
            self.content.as_widget().tag()
        }

        fn state(&self) -> widget::tree::State {
            self.content.as_widget().state()
        }

        fn children(&self) -> Vec<widget::Tree> {
            self.content.as_widget().children()
        }

        fn diff(&self, tree: &mut widget::Tree) {
            self.content.as_widget().diff(tree);
        }

        fn size(&self) -> Size<Length> {
            self.content.as_widget().size()
        }

        fn layout(
            &self,
            tree: &mut widget::Tree,
            renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            self.content.as_widget().layout(tree, renderer, limits)
        }

        fn draw(
            &self,
            tree: &widget::Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            let bounds = layout.bounds();

            if let Some(position) = cursor.position_in(bounds) {
                renderer.with_layer(bounds, |renderer| {
                    renderer.with_transformation(
                        Transformation::translate(
                            bounds.x + position.x * (1.0 - self.zoom),
                            bounds.y + position.y * (1.0 - self.zoom),
                        ) * Transformation::scale(self.zoom)
                            * Transformation::translate(-bounds.x, -bounds.y),
                        |renderer| {
                            self.content.as_widget().draw(
                                tree,
                                renderer,
                                theme,
                                style,
                                layout,
                                mouse::Cursor::Unavailable,
                                viewport,
                            );
                        },
                    );
                });
            } else {
                self.content.as_widget().draw(
                    tree, renderer, theme, style, layout, cursor, viewport,
                );
            }
        }

        fn mouse_interaction(
            &self,
            _state: &widget::Tree,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            _viewport: &Rectangle,
            _renderer: &Renderer,
        ) -> mouse::Interaction {
            if cursor.is_over(layout.bounds()) {
                mouse::Interaction::ZoomIn
            } else {
                mouse::Interaction::None
            }
        }
    }

    impl<'a, Message> From<Loupe<'a, Message>>
        for Element<'a, Message, Theme, Renderer>
    where
        Message: 'a,
    {
        fn from(loupe: Loupe<'a, Message>) -> Self {
            Self::new(loupe)
        }
    }
}
