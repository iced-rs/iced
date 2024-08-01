use iced::alignment;
use iced::mouse;
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path};
use iced::widget::{column, row, text, Slider};
use iced::{
    Center, Color, Element, Fill, Font, Pixels, Point, Rectangle, Renderer,
    Size, Vector,
};
use palette::{convert::FromColor, rgb::Rgb, Darken, Hsl, Lighten, ShiftHue};
use std::marker::PhantomData;
use std::ops::RangeInclusive;

pub fn main() -> iced::Result {
    iced::application(
        "Color Palette - Iced",
        ColorPalette::update,
        ColorPalette::view,
    )
    .theme(ColorPalette::theme)
    .default_font(Font::MONOSPACE)
    .antialiasing(true)
    .run()
}

#[derive(Default)]
pub struct ColorPalette {
    theme: Theme,
    rgb: ColorPicker<Color>,
    hsl: ColorPicker<palette::Hsl>,
    hsv: ColorPicker<palette::Hsv>,
    hwb: ColorPicker<palette::Hwb>,
    lab: ColorPicker<palette::Lab>,
    lch: ColorPicker<palette::Lch>,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    RgbColorChanged(Color),
    HslColorChanged(palette::Hsl),
    HsvColorChanged(palette::Hsv),
    HwbColorChanged(palette::Hwb),
    LabColorChanged(palette::Lab),
    LchColorChanged(palette::Lch),
}

impl ColorPalette {
    fn update(&mut self, message: Message) {
        let srgb = match message {
            Message::RgbColorChanged(rgb) => Rgb::from(rgb),
            Message::HslColorChanged(hsl) => Rgb::from_color(hsl),
            Message::HsvColorChanged(hsv) => Rgb::from_color(hsv),
            Message::HwbColorChanged(hwb) => Rgb::from_color(hwb),
            Message::LabColorChanged(lab) => Rgb::from_color(lab),
            Message::LchColorChanged(lch) => Rgb::from_color(lch),
        };

        self.theme = Theme::new(srgb);
    }

    fn view(&self) -> Element<Message> {
        let base = self.theme.base;

        let srgb = Rgb::from(base);
        let hsl = palette::Hsl::from_color(srgb);
        let hsv = palette::Hsv::from_color(srgb);
        let hwb = palette::Hwb::from_color(srgb);
        let lab = palette::Lab::from_color(srgb);
        let lch = palette::Lch::from_color(srgb);

        column![
            self.rgb.view(base).map(Message::RgbColorChanged),
            self.hsl.view(hsl).map(Message::HslColorChanged),
            self.hsv.view(hsv).map(Message::HsvColorChanged),
            self.hwb.view(hwb).map(Message::HwbColorChanged),
            self.lab.view(lab).map(Message::LabColorChanged),
            self.lch.view(lch).map(Message::LchColorChanged),
            self.theme.view(),
        ]
        .padding(10)
        .spacing(10)
        .into()
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::custom(
            String::from("Custom"),
            iced::theme::Palette {
                background: self.theme.base,
                primary: *self.theme.lower.first().unwrap(),
                text: *self.theme.higher.last().unwrap(),
                success: *self.theme.lower.last().unwrap(),
                danger: *self.theme.higher.last().unwrap(),
            },
        )
    }
}

#[derive(Debug)]
struct Theme {
    lower: Vec<Color>,
    base: Color,
    higher: Vec<Color>,
    canvas_cache: canvas::Cache,
}

impl Theme {
    pub fn new(base: impl Into<Color>) -> Theme {
        let base = base.into();

        // Convert to HSL color for manipulation
        let hsl = Hsl::from_color(Rgb::from(base));

        let lower = [
            hsl.shift_hue(-135.0).lighten(0.075),
            hsl.shift_hue(-120.0),
            hsl.shift_hue(-105.0).darken(0.075),
            hsl.darken(0.075),
        ];

        let higher = [
            hsl.lighten(0.075),
            hsl.shift_hue(105.0).darken(0.075),
            hsl.shift_hue(120.0),
            hsl.shift_hue(135.0).lighten(0.075),
        ];

        Theme {
            lower: lower
                .iter()
                .map(|&color| Rgb::from_color(color).into())
                .collect(),
            base,
            higher: higher
                .iter()
                .map(|&color| Rgb::from_color(color).into())
                .collect(),
            canvas_cache: canvas::Cache::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.lower.len() + self.higher.len() + 1
    }

    pub fn colors(&self) -> impl Iterator<Item = &Color> {
        self.lower
            .iter()
            .chain(std::iter::once(&self.base))
            .chain(self.higher.iter())
    }

    pub fn view(&self) -> Element<Message> {
        Canvas::new(self).width(Fill).height(Fill).into()
    }

    fn draw(&self, frame: &mut Frame, text_color: Color) {
        let pad = 20.0;

        let box_size = Size {
            width: frame.width() / self.len() as f32,
            height: frame.height() / 2.0 - pad,
        };

        let triangle = Path::new(|path| {
            path.move_to(Point { x: 0.0, y: -0.5 });
            path.line_to(Point { x: -0.5, y: 0.0 });
            path.line_to(Point { x: 0.5, y: 0.0 });
            path.close();
        });

        let mut text = canvas::Text {
            horizontal_alignment: alignment::Horizontal::Center,
            vertical_alignment: alignment::Vertical::Top,
            size: Pixels(15.0),
            color: text_color,
            ..canvas::Text::default()
        };

        for (i, &color) in self.colors().enumerate() {
            let anchor = Point {
                x: (i as f32) * box_size.width,
                y: 0.0,
            };
            frame.fill_rectangle(anchor, box_size, color);

            // We show a little indicator for the base color
            if color == self.base {
                let triangle_x = anchor.x + box_size.width / 2.0;

                frame.with_save(|frame| {
                    frame.translate(Vector::new(triangle_x, 0.0));
                    frame.scale(10.0);
                    frame.rotate(std::f32::consts::PI);

                    frame.fill(&triangle, Color::WHITE);
                });

                frame.with_save(|frame| {
                    frame.translate(Vector::new(triangle_x, box_size.height));
                    frame.scale(10.0);

                    frame.fill(&triangle, Color::WHITE);
                });
            }

            frame.fill_text(canvas::Text {
                content: color_hex_string(&color),
                position: Point {
                    x: anchor.x + box_size.width / 2.0,
                    y: box_size.height,
                },
                ..text
            });
        }

        text.vertical_alignment = alignment::Vertical::Bottom;

        let hsl = Hsl::from_color(Rgb::from(self.base));
        for i in 0..self.len() {
            let pct = (i as f32 + 1.0) / (self.len() as f32 + 1.0);
            let graded = Hsl {
                lightness: 1.0 - pct,
                ..hsl
            };
            let color: Color = Rgb::from_color(graded).into();

            let anchor = Point {
                x: (i as f32) * box_size.width,
                y: box_size.height + 2.0 * pad,
            };

            frame.fill_rectangle(anchor, box_size, color);

            frame.fill_text(canvas::Text {
                content: color_hex_string(&color),
                position: Point {
                    x: anchor.x + box_size.width / 2.0,
                    y: box_size.height + 2.0 * pad,
                },
                ..text
            });
        }
    }
}

impl<Message> canvas::Program<Message> for Theme {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let theme = self.canvas_cache.draw(renderer, bounds.size(), |frame| {
            let palette = theme.extended_palette();

            self.draw(frame, palette.background.base.text);
        });

        vec![theme]
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::new(Color::from_rgb8(75, 128, 190))
    }
}

fn color_hex_string(color: &Color) -> String {
    format!(
        "#{:x}{:x}{:x}",
        (255.0 * color.r).round() as u8,
        (255.0 * color.g).round() as u8,
        (255.0 * color.b).round() as u8
    )
}

#[derive(Default)]
struct ColorPicker<C: ColorSpace> {
    color_space: PhantomData<C>,
}

trait ColorSpace: Sized {
    const LABEL: &'static str;
    const COMPONENT_RANGES: [RangeInclusive<f64>; 3];

    fn new(a: f32, b: f32, c: f32) -> Self;

    fn components(&self) -> [f32; 3];

    fn to_string(&self) -> String;
}

impl<C: ColorSpace + Copy> ColorPicker<C> {
    fn view(&self, color: C) -> Element<C> {
        let [c1, c2, c3] = color.components();
        let [cr1, cr2, cr3] = C::COMPONENT_RANGES;

        fn slider<'a, C: Clone>(
            range: RangeInclusive<f64>,
            component: f32,
            update: impl Fn(f32) -> C + 'a,
        ) -> Slider<'a, f64, C> {
            Slider::new(range, f64::from(component), move |v| update(v as f32))
                .step(0.01)
        }

        row![
            text(C::LABEL).width(50),
            slider(cr1, c1, move |v| C::new(v, c2, c3)),
            slider(cr2, c2, move |v| C::new(c1, v, c3)),
            slider(cr3, c3, move |v| C::new(c1, c2, v)),
            text(color.to_string()).width(185).size(12),
        ]
        .spacing(10)
        .align_y(Center)
        .into()
    }
}

impl ColorSpace for Color {
    const LABEL: &'static str = "RGB";
    const COMPONENT_RANGES: [RangeInclusive<f64>; 3] =
        [0.0..=1.0, 0.0..=1.0, 0.0..=1.0];

    fn new(r: f32, g: f32, b: f32) -> Self {
        Color::from_rgb(r, g, b)
    }

    fn components(&self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }

    fn to_string(&self) -> String {
        format!(
            "rgb({:.0}, {:.0}, {:.0})",
            255.0 * self.r,
            255.0 * self.g,
            255.0 * self.b
        )
    }
}

impl ColorSpace for palette::Hsl {
    const LABEL: &'static str = "HSL";
    const COMPONENT_RANGES: [RangeInclusive<f64>; 3] =
        [0.0..=360.0, 0.0..=1.0, 0.0..=1.0];

    fn new(hue: f32, saturation: f32, lightness: f32) -> Self {
        palette::Hsl::new(
            palette::RgbHue::from_degrees(hue),
            saturation,
            lightness,
        )
    }

    fn components(&self) -> [f32; 3] {
        [
            self.hue.into_positive_degrees(),
            self.saturation,
            self.lightness,
        ]
    }

    fn to_string(&self) -> String {
        format!(
            "hsl({:.1}, {:.1}%, {:.1}%)",
            self.hue.into_positive_degrees(),
            100.0 * self.saturation,
            100.0 * self.lightness
        )
    }
}

impl ColorSpace for palette::Hsv {
    const LABEL: &'static str = "HSV";
    const COMPONENT_RANGES: [RangeInclusive<f64>; 3] =
        [0.0..=360.0, 0.0..=1.0, 0.0..=1.0];

    fn new(hue: f32, saturation: f32, value: f32) -> Self {
        palette::Hsv::new(palette::RgbHue::from_degrees(hue), saturation, value)
    }

    fn components(&self) -> [f32; 3] {
        [
            self.hue.into_positive_degrees(),
            self.saturation,
            self.value,
        ]
    }

    fn to_string(&self) -> String {
        format!(
            "hsv({:.1}, {:.1}%, {:.1}%)",
            self.hue.into_positive_degrees(),
            100.0 * self.saturation,
            100.0 * self.value
        )
    }
}

impl ColorSpace for palette::Hwb {
    const LABEL: &'static str = "HWB";
    const COMPONENT_RANGES: [RangeInclusive<f64>; 3] =
        [0.0..=360.0, 0.0..=1.0, 0.0..=1.0];

    fn new(hue: f32, whiteness: f32, blackness: f32) -> Self {
        palette::Hwb::new(
            palette::RgbHue::from_degrees(hue),
            whiteness,
            blackness,
        )
    }

    fn components(&self) -> [f32; 3] {
        [
            self.hue.into_positive_degrees(),
            self.whiteness,
            self.blackness,
        ]
    }

    fn to_string(&self) -> String {
        format!(
            "hwb({:.1}, {:.1}%, {:.1}%)",
            self.hue.into_positive_degrees(),
            100.0 * self.whiteness,
            100.0 * self.blackness
        )
    }
}

impl ColorSpace for palette::Lab {
    const LABEL: &'static str = "Lab";
    const COMPONENT_RANGES: [RangeInclusive<f64>; 3] =
        [0.0..=100.0, -128.0..=127.0, -128.0..=127.0];

    fn new(l: f32, a: f32, b: f32) -> Self {
        palette::Lab::new(l, a, b)
    }

    fn components(&self) -> [f32; 3] {
        [self.l, self.a, self.b]
    }

    fn to_string(&self) -> String {
        format!("Lab({:.1}, {:.1}, {:.1})", self.l, self.a, self.b)
    }
}

impl ColorSpace for palette::Lch {
    const LABEL: &'static str = "Lch";
    const COMPONENT_RANGES: [RangeInclusive<f64>; 3] =
        [0.0..=100.0, 0.0..=128.0, 0.0..=360.0];

    fn new(l: f32, chroma: f32, hue: f32) -> Self {
        palette::Lch::new(l, chroma, palette::LabHue::from_degrees(hue))
    }

    fn components(&self) -> [f32; 3] {
        [self.l, self.chroma, self.hue.into_positive_degrees()]
    }

    fn to_string(&self) -> String {
        format!(
            "Lch({:.1}, {:.1}, {:.1})",
            self.l,
            self.chroma,
            self.hue.into_positive_degrees()
        )
    }
}
