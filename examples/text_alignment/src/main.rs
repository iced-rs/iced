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
                widget_text("Horizontal Alignment"),
                case(
                    "Left Aligned:",
                    text::Alignment::Left,
                    iced::alignment::Vertical::Top,
                    100,
                    50,
                ),
                case(
                    "Right Aligned:",
                    text::Alignment::Right,
                    iced::alignment::Vertical::Top,
                    100,
                    50,
                ),
                case(
                    "Center Aligned:",
                    text::Alignment::Center,
                    iced::alignment::Vertical::Top,
                    100,
                    50,
                ),
                widget_text("Vertical Alignment"),
                case(
                    "Top Aligned:",
                    text::Alignment::Center,
                    iced::alignment::Vertical::Top,
                    50,
                    100,
                ),
                case(
                    "Bottom Aligned:",
                    text::Alignment::Center,
                    iced::alignment::Vertical::Bottom,
                    50,
                    100,
                ),
                case(
                    "Center Aligned:",
                    text::Alignment::Center,
                    iced::alignment::Vertical::Center,
                    50,
                    100,
                ),
            ]
            .spacing(10)
            .padding(20),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }
}

fn case<'a>(
    label: &'static str,
    align_x: text::Alignment,
    align_y: iced::alignment::Vertical,
    width: impl Into<Length>,
    height: impl Into<Length>,
) -> Element<'a, Message> {
    column![
        widget_text(label),
        container(CustomText {
            content: "10".to_string(),
            align_x,
            align_y,
        })
        .width(width)
        .height(height)
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
    align_x: text::Alignment,
    align_y: iced::alignment::Vertical,
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
                align_x: self.align_x,
                align_y: self.align_y,
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
