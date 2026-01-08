use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::text::{self, Text};
use iced::advanced::widget::{self, Widget};
use iced::widget::{column, container, text as widget_text};
use iced::{Color, Element, Font, Length, Pixels, Rectangle, Size, Task, Theme};

pub fn main() -> iced::Result {
    iced::application(State::new, State::update, State::view)
        .title(|_: &State| "Text Alignment".to_string())
        .run()
}

struct State;

#[derive(Debug, Clone, Copy)]
enum Message {}

impl State {
    fn new() -> (Self, Task<Message>) {
        (State, Task::none())
    }

    fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        container(
            column![
                case("Left Aligned:", text::Alignment::Left),
                case("Right Aligned:", text::Alignment::Right),
                case("Center Aligned:", text::Alignment::Center),
            ]
            .spacing(10)
            .padding(20),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }
}

fn case<'a>(label: &'static str, alignment: text::Alignment) -> Element<'a, Message> {
    column![
        widget_text(label),
        container(CustomText {
            content: "10".to_string(),
            alignment,
        })
        .width(50)
        .height(30)
        .style(|_| container::Style {
            border: iced::Border {
                color: Color::BLACK,
                width: 1.0,
                radius: 0.0.into()
            },
            ..Default::default()
        })
    ]
    .into()
}

struct CustomText {
    content: String,
    alignment: text::Alignment,
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for CustomText
where
    Renderer: text::Renderer<Font = Font>,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.max())
    }

    fn draw(
        &self,
        _tree: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: iced::mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                snap: true,
            },
            Color::from_rgb(0.9, 0.9, 0.9),
        );

        renderer.fill_text(
            Text {
                content: self.content.clone(),
                bounds: bounds.size(),
                size: Pixels(20.0),
                line_height: text::LineHeight::default(),
                font: Font::MONOSPACE,
                align_x: self.alignment,
                align_y: iced::alignment::Vertical::Top,
                shaping: text::Shaping::Basic,
                wrapping: text::Wrapping::Glyph,
                hint_factor: None,
            },
            bounds.position(),
            Color::BLACK,
            *viewport,
        );
    }
}

impl<'a, Message, Renderer> From<CustomText> for Element<'a, Message, Theme, Renderer>
where
    Renderer: text::Renderer<Font = Font>,
{
    fn from(widget: CustomText) -> Self {
        Self::new(widget)
    }
}
