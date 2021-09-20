use iced::canvas::{self, Cursor, Frame, Geometry, Path};
use iced::{
    alignment, slider, Alignment, Canvas, Color, Column, Element, Length,
    Point, Rectangle, Row, Sandbox, Settings, Size, Slider, Text, Vector,
};
use palette::{self, Hsl, Limited, Srgb};
use std::marker::PhantomData;
use std::ops::RangeInclusive;

pub fn main() -> iced::Result {
    ColorPalette::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
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

impl Sandbox for ColorPalette {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Color palette - Iced")
    }

    fn update(&mut self, message: Message) {
        let srgb = match message {
            Message::RgbColorChanged(rgb) => palette::Srgb::from(rgb),
            Message::HslColorChanged(hsl) => palette::Srgb::from(hsl),
            Message::HsvColorChanged(hsv) => palette::Srgb::from(hsv),
            Message::HwbColorChanged(hwb) => palette::Srgb::from(hwb),
            Message::LabColorChanged(lab) => palette::Srgb::from(lab),
            Message::LchColorChanged(lch) => palette::Srgb::from(lch),
        };

        self.theme = Theme::new(srgb.clamp());
    }

    fn view(&mut self) -> Element<Message> {
        let base = self.theme.base;

        let srgb = palette::Srgb::from(base);
        let hsl = palette::Hsl::from(srgb);
        let hsv = palette::Hsv::from(srgb);
        let hwb = palette::Hwb::from(srgb);
        let lab = palette::Lab::from(srgb);
        let lch = palette::Lch::from(srgb);

        Column::new()
            .padding(10)
            .spacing(10)
            .push(self.rgb.view(base).map(Message::RgbColorChanged))
            .push(self.hsl.view(hsl).map(Message::HslColorChanged))
            .push(self.hsv.view(hsv).map(Message::HsvColorChanged))
            .push(self.hwb.view(hwb).map(Message::HwbColorChanged))
            .push(self.lab.view(lab).map(Message::LabColorChanged))
            .push(self.lch.view(lch).map(Message::LchColorChanged))
            .push(self.theme.view())
            .into()
    }
}

#[derive(Debug)]
pub struct Theme {
    lower: Vec<Color>,
    base: Color,
    higher: Vec<Color>,
    canvas_cache: canvas::Cache,
}

impl Theme {
    pub fn new(base: impl Into<Color>) -> Theme {
        use palette::{Hue, Shade};

        let base = base.into();

        // Convert to HSL color for manipulation
        let hsl = Hsl::from(Srgb::from(base));

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
                .map(|&color| Srgb::from(color).clamp().into())
                .collect(),
            base,
            higher: higher
                .iter()
                .map(|&color| Srgb::from(color).clamp().into())
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

    pub fn view(&mut self) -> Element<Message> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn draw(&self, frame: &mut Frame) {
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
            size: 15.0,
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

        let hsl = Hsl::from(Srgb::from(self.base));
        for i in 0..self.len() {
            let pct = (i as f32 + 1.0) / (self.len() as f32 + 1.0);
            let graded = Hsl {
                lightness: 1.0 - pct,
                ..hsl
            };
            let color: Color = Srgb::from(graded.clamp()).into();

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

impl canvas::Program<Message> for Theme {
    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let theme = self.canvas_cache.draw(bounds.size(), |frame| {
            self.draw(frame);
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
    sliders: [slider::State; 3],
    color_space: PhantomData<C>,
}

trait ColorSpace: Sized {
    const LABEL: &'static str;
    const COMPONENT_RANGES: [RangeInclusive<f64>; 3];

    fn new(a: f32, b: f32, c: f32) -> Self;

    fn components(&self) -> [f32; 3];

    fn to_string(&self) -> String;
}

impl<C: 'static + ColorSpace + Copy> ColorPicker<C> {
    fn view(&mut self, color: C) -> Element<C> {
        let [c1, c2, c3] = color.components();
        let [s1, s2, s3] = &mut self.sliders;
        let [cr1, cr2, cr3] = C::COMPONENT_RANGES;

        fn slider<C: Clone>(
            state: &mut slider::State,
            range: RangeInclusive<f64>,
            component: f32,
            update: impl Fn(f32) -> C + 'static,
        ) -> Slider<f64, C> {
            Slider::new(state, range, f64::from(component), move |v| {
                update(v as f32)
            })
            .step(0.01)
        }

        Row::new()
            .spacing(10)
            .align_items(Alignment::Center)
            .push(Text::new(C::LABEL).width(Length::Units(50)))
            .push(slider(s1, cr1, c1, move |v| C::new(v, c2, c3)))
            .push(slider(s2, cr2, c2, move |v| C::new(c1, v, c3)))
            .push(slider(s3, cr3, c3, move |v| C::new(c1, c2, v)))
            .push(
                Text::new(color.to_string())
                    .width(Length::Units(185))
                    .size(14),
            )
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
            self.hue.to_positive_degrees(),
            self.saturation,
            self.lightness,
        ]
    }

    fn to_string(&self) -> String {
        format!(
            "hsl({:.1}, {:.1}%, {:.1}%)",
            self.hue.to_positive_degrees(),
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
        [self.hue.to_positive_degrees(), self.saturation, self.value]
    }

    fn to_string(&self) -> String {
        format!(
            "hsv({:.1}, {:.1}%, {:.1}%)",
            self.hue.to_positive_degrees(),
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
            self.hue.to_positive_degrees(),
            self.whiteness,
            self.blackness,
        ]
    }

    fn to_string(&self) -> String {
        format!(
            "hwb({:.1}, {:.1}%, {:.1}%)",
            self.hue.to_positive_degrees(),
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
        [self.l, self.chroma, self.hue.to_positive_degrees()]
    }

    fn to_string(&self) -> String {
        format!(
            "Lch({:.1}, {:.1}, {:.1})",
            self.l,
            self.chroma,
            self.hue.to_positive_degrees()
        )
    }
}
