use iced::alignment;
use iced::mouse;
use iced::widget::{
    canvas, checkbox, column, horizontal_space, row, slider, text,
};
use iced::{Center, Element, Fill, Point, Rectangle, Renderer, Theme, Vector};

pub fn main() -> iced::Result {
    iced::application(
        "Vectorial Text - Iced",
        VectorialText::update,
        VectorialText::view,
    )
    .theme(|_| Theme::Dark)
    .antialiasing(true)
    .run()
}

#[derive(Default)]
struct VectorialText {
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    SizeChanged(f32),
    AngleChanged(f32),
    ScaleChanged(f32),
    ToggleJapanese(bool),
}

impl VectorialText {
    fn update(&mut self, message: Message) {
        match message {
            Message::SizeChanged(size) => {
                self.state.size = size;
            }
            Message::AngleChanged(angle) => {
                self.state.angle = angle;
            }
            Message::ScaleChanged(scale) => {
                self.state.scale = scale;
            }
            Message::ToggleJapanese(use_japanese) => {
                self.state.use_japanese = use_japanese;
            }
        }

        self.state.cache.clear();
    }

    fn view(&self) -> Element<Message> {
        let slider_with_label = |label, range, value, message: fn(f32) -> _| {
            column![
                row![text(label), horizontal_space(), text!("{:.2}", value)],
                slider(range, value, message).step(0.01)
            ]
            .spacing(2)
        };

        column![
            canvas(&self.state).width(Fill).height(Fill),
            column![
                checkbox("Use Japanese", self.state.use_japanese,)
                    .on_toggle(Message::ToggleJapanese),
                row![
                    slider_with_label(
                        "Size",
                        2.0..=80.0,
                        self.state.size,
                        Message::SizeChanged,
                    ),
                    slider_with_label(
                        "Angle",
                        0.0..=360.0,
                        self.state.angle,
                        Message::AngleChanged,
                    ),
                    slider_with_label(
                        "Scale",
                        1.0..=20.0,
                        self.state.scale,
                        Message::ScaleChanged,
                    ),
                ]
                .spacing(20),
            ]
            .align_x(Center)
            .spacing(10)
        ]
        .spacing(10)
        .padding(20)
        .into()
    }
}

struct State {
    size: f32,
    angle: f32,
    scale: f32,
    use_japanese: bool,
    cache: canvas::Cache,
}

impl State {
    pub fn new() -> Self {
        Self {
            size: 40.0,
            angle: 0.0,
            scale: 1.0,
            use_japanese: false,
            cache: canvas::Cache::new(),
        }
    }
}

impl<Message> canvas::Program<Message> for State {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let palette = theme.palette();
            let center = bounds.center();

            frame.translate(Vector::new(center.x, center.y));
            frame.scale(self.scale);
            frame.rotate(self.angle * std::f32::consts::PI / 180.0);

            frame.fill_text(canvas::Text {
                position: Point::new(0.0, 0.0),
                color: palette.text,
                size: self.size.into(),
                content: String::from(if self.use_japanese {
                    "ベクトルテキスト🎉"
                } else {
                    "Vectorial Text! 🎉"
                }),
                horizontal_alignment: alignment::Horizontal::Center,
                vertical_alignment: alignment::Vertical::Center,
                shaping: text::Shaping::Advanced,
                ..canvas::Text::default()
            });
        });

        vec![geometry]
    }
}

impl Default for State {
    fn default() -> Self {
        State::new()
    }
}
