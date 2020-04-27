use iced::{
    canvas, slider, Canvas, Color, Column, Element, Length, Row, Sandbox,
    Settings, Slider, Text,
};
use palette::{self, Limited};
use std::marker::PhantomData;
use std::ops::RangeInclusive;

pub fn main() {
    ColorPalette::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

#[derive(Debug, Default)]
pub struct State {
    color: Color,
    theme: Vec<Color>,
}

fn generate_theme(base_color: &Color) -> Vec<Color> {
    use palette::{Hsl, Hue, Shade, Srgb};
    let mut theme = Vec::<Color>::new();
    // Convert to HSL color for manipulation
    let hsl = Hsl::from(Srgb::from(*base_color));

    theme.push(
        Srgb::from(hsl.shift_hue(-135.0).lighten(0.075))
            .clamp()
            .into(),
    );
    theme.push(Srgb::from(hsl.shift_hue(-120.0)).clamp().into());
    theme.push(
        Srgb::from(hsl.shift_hue(-105.0).darken(0.075))
            .clamp()
            .into(),
    );
    theme.push(Srgb::from(hsl.darken(0.075)).clamp().into());
    theme.push(*base_color);
    theme.push(Srgb::from(hsl.lighten(0.075)).clamp().into());
    theme.push(
        Srgb::from(hsl.shift_hue(105.0).darken(0.075))
            .clamp()
            .into(),
    );
    theme.push(Srgb::from(hsl.shift_hue(120.0)).clamp().into());
    theme.push(
        Srgb::from(hsl.shift_hue(135.0).lighten(0.075))
            .clamp()
            .into(),
    );
    theme
}

struct ColorPicker<C: ColorSpace> {
    sliders: [slider::State; 3],
    color_space: PhantomData<C>,
}

trait ColorSpace: Sized {
    const LABEL: &'static str;
    const COMPONENT_RANGES: [RangeInclusive<f32>; 3];

    fn new(a: f32, b: f32, c: f32) -> Self;

    fn components(&self) -> [f32; 3];

    fn update_component(c: Self, i: usize, val: f32) -> Self;

    fn to_string(&self) -> String;
}

impl<C: 'static + ColorSpace + Copy> ColorPicker<C> {
    fn view(&mut self, color: C) -> Element<C> {
        let [c1, c2, c3] = color.components();
        let [s1, s2, s3] = &mut self.sliders;
        let [cr1, cr2, cr3] = C::COMPONENT_RANGES;
        Row::new()
            .spacing(10)
            .push(Text::new(C::LABEL).width(Length::Units(50)))
            .push(Slider::new(s1, cr1, c1, move |v| {
                C::update_component(color, 0, v)
            }))
            .push(Slider::new(s2, cr2, c2, move |v| {
                C::update_component(color, 1, v)
            }))
            .push(Slider::new(s3, cr3, c3, move |v| {
                C::update_component(color, 2, v)
            }))
            .push(
                Text::new(color.to_string())
                    .width(Length::Units(185))
                    .size(16),
            )
            .into()
    }
}

impl ColorSpace for Color {
    const LABEL: &'static str = "RGB";
    const COMPONENT_RANGES: [RangeInclusive<f32>; 3] =
        [0.0..=1.0, 0.0..=1.0, 0.0..=1.0];

    fn new(r: f32, g: f32, b: f32) -> Self {
        Color::from_rgb(r, g, b)
    }

    fn components(&self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }

    fn update_component(c: Color, i: usize, val: f32) -> Self {
        match i {
            0 => Color { r: val, ..c },
            1 => Color { g: val, ..c },
            2 => Color { b: val, ..c },
            _ => panic!("Invalid component index: {:?}", i),
        }
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
    const COMPONENT_RANGES: [RangeInclusive<f32>; 3] =
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

    fn update_component(c: palette::Hsl, i: usize, val: f32) -> Self {
        match i {
            0 => palette::Hsl {
                hue: palette::RgbHue::from_degrees(val),
                ..c
            },
            1 => palette::Hsl {
                saturation: val,
                ..c
            },
            2 => palette::Hsl {
                lightness: val,
                ..c
            },
            _ => panic!("Invalid component index: {:?}", i),
        }
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
    const COMPONENT_RANGES: [RangeInclusive<f32>; 3] =
        [0.0..=360.0, 0.0..=1.0, 0.0..=1.0];

    fn new(hue: f32, saturation: f32, value: f32) -> Self {
        palette::Hsv::new(palette::RgbHue::from_degrees(hue), saturation, value)
    }

    fn components(&self) -> [f32; 3] {
        [self.hue.to_positive_degrees(), self.saturation, self.value]
    }

    fn update_component(c: palette::Hsv, i: usize, val: f32) -> Self {
        match i {
            0 => palette::Hsv {
                hue: palette::RgbHue::from_degrees(val),
                ..c
            },
            1 => palette::Hsv {
                saturation: val,
                ..c
            },
            2 => palette::Hsv { value: val, ..c },
            _ => panic!("Invalid component index: {:?}", i),
        }
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
    const COMPONENT_RANGES: [RangeInclusive<f32>; 3] =
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

    fn update_component(c: palette::Hwb, i: usize, val: f32) -> Self {
        match i {
            0 => palette::Hwb {
                hue: palette::RgbHue::from_degrees(val),
                ..c
            },
            1 => palette::Hwb {
                whiteness: val,
                ..c
            },
            2 => palette::Hwb {
                blackness: val,
                ..c
            },
            _ => panic!("Invalid component index: {:?}", i),
        }
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
    const COMPONENT_RANGES: [RangeInclusive<f32>; 3] =
        [0.0..=100.0, -128.0..=127.0, -128.0..=127.0];

    fn new(l: f32, a: f32, b: f32) -> Self {
        palette::Lab::new(l, a, b)
    }

    fn components(&self) -> [f32; 3] {
        [self.l, self.a, self.b]
    }

    fn update_component(c: palette::Lab, i: usize, val: f32) -> Self {
        match i {
            0 => palette::Lab { l: val, ..c },
            1 => palette::Lab { a: val, ..c },
            2 => palette::Lab { b: val, ..c },
            _ => panic!("Invalid component index: {:?}", i),
        }
    }

    fn to_string(&self) -> String {
        format!("Lab({:.1}, {:.1}, {:.1})", self.l, self.a, self.b)
    }
}

impl ColorSpace for palette::Lch {
    const LABEL: &'static str = "Lch";
    const COMPONENT_RANGES: [RangeInclusive<f32>; 3] =
        [0.0..=100.0, 0.0..=128.0, 0.0..=360.0];

    fn new(l: f32, chroma: f32, hue: f32) -> Self {
        palette::Lch::new(l, chroma, palette::LabHue::from_degrees(hue))
    }

    fn components(&self) -> [f32; 3] {
        [self.l, self.chroma, self.hue.to_positive_degrees()]
    }

    fn update_component(c: palette::Lch, i: usize, val: f32) -> Self {
        match i {
            0 => palette::Lch { l: val, ..c },
            1 => palette::Lch { chroma: val, ..c },
            2 => palette::Lch {
                hue: palette::LabHue::from_degrees(val),
                ..c
            },
            _ => panic!("Invalid component index: {:?}", i),
        }
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

pub struct ColorPalette {
    state: State,
    rgb: ColorPicker<Color>,
    hsl: ColorPicker<palette::Hsl>,
    hsv: ColorPicker<palette::Hsv>,
    hwb: ColorPicker<palette::Hwb>,
    lab: ColorPicker<palette::Lab>,
    lch: ColorPicker<palette::Lch>,
    canvas_layer: canvas::layer::Cache<State>,
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
        fn triple_slider() -> [slider::State; 3] {
            [
                slider::State::new(),
                slider::State::new(),
                slider::State::new(),
            ]
        }

        ColorPalette {
            state: State::new(),
            rgb: ColorPicker {
                sliders: triple_slider(),
                color_space: PhantomData::<Color>,
            },
            hsl: ColorPicker {
                sliders: triple_slider(),
                color_space: PhantomData::<palette::Hsl>,
            },
            hsv: ColorPicker {
                sliders: triple_slider(),
                color_space: PhantomData::<palette::Hsv>,
            },

            hwb: ColorPicker {
                sliders: triple_slider(),
                color_space: PhantomData::<palette::Hwb>,
            },
            lab: ColorPicker {
                sliders: triple_slider(),
                color_space: PhantomData::<palette::Lab>,
            },
            lch: ColorPicker {
                sliders: triple_slider(),
                color_space: PhantomData::<palette::Lch>,
            },
            canvas_layer: canvas::layer::Cache::new(),
        }
    }

    fn title(&self) -> String {
        String::from("Color Palette")
    }

    fn update(&mut self, message: Message) {
        let mut srgb = match message {
            Message::RgbColorChanged(rgb) => palette::Srgb::from(rgb),
            Message::HslColorChanged(hsl) => palette::Srgb::from(hsl),
            Message::HsvColorChanged(hsv) => palette::Srgb::from(hsv),
            Message::HwbColorChanged(hwb) => palette::Srgb::from(hwb),
            Message::LabColorChanged(lab) => palette::Srgb::from(lab),
            Message::LchColorChanged(lch) => palette::Srgb::from(lch),
        };
        srgb.clamp_self();
        self.canvas_layer.clear();
        self.state.color = Color::from(srgb);

        // Set theme colors
        self.state.theme = generate_theme(&self.state.color);
    }

    fn view(&mut self) -> Element<Message> {
        let color = self.state.color;
        let srgb = palette::Srgb::from(self.state.color);
        let hsl = palette::Hsl::from(srgb);
        let hsv = palette::Hsv::from(srgb);
        let hwb = palette::Hwb::from(srgb);
        let lab = palette::Lab::from(srgb);
        let lch = palette::Lch::from(srgb);

        Column::new()
            .padding(10)
            .spacing(10)
            .push(self.rgb.view(color).map(Message::RgbColorChanged))
            .push(self.hsl.view(hsl).map(Message::HslColorChanged))
            .push(self.hsv.view(hsv).map(Message::HsvColorChanged))
            .push(self.hwb.view(hwb).map(Message::HwbColorChanged))
            .push(self.lab.view(lab).map(Message::LabColorChanged))
            .push(self.lch.view(lch).map(Message::LchColorChanged))
            .push(
                Canvas::new()
                    .width(Length::Fill)
                    // .height(Length::Units(250))
                    .height(Length::Fill)
                    .push(self.canvas_layer.with(&self.state)),
            )
            .into()
    }
}

impl State {
    pub fn new() -> State {
        let base = Color::from_rgb8(75, 128, 190);
        State {
            color: base,
            theme: generate_theme(&base),
        }
    }
}

impl canvas::Drawable for State {
    fn draw(&self, frame: &mut canvas::Frame) {
        use canvas::{Fill, Path};
        use iced::{HorizontalAlignment, VerticalAlignment};
        use iced_native::{Point, Size};
        use palette::{Hsl, Srgb};

        if self.theme.len() == 0 {
            return;
        }

        let pad = 20.0;

        let box_size = Size {
            width: frame.width() / self.theme.len() as f32,
            height: frame.height() / 2.0 - pad,
        };

        let mut text = canvas::Text::default();
        text.horizontal_alignment = HorizontalAlignment::Center;
        text.vertical_alignment = VerticalAlignment::Top;
        text.size = 15.0;

        for i in 0..self.theme.len() {
            let anchor = Point {
                x: (i as f32) * box_size.width,
                y: 0.0,
            };
            let rect = Path::new(|path| {
                path.rectangle(anchor, box_size);
            });
            frame.fill(&rect, Fill::Color(self.theme[i]));

            if self.theme[i] == self.color {
                let cx = anchor.x + box_size.width / 2.0;
                let tri_w = 10.0;

                let tri = Path::new(|path| {
                    path.move_to(Point {
                        x: cx - tri_w,
                        y: 0.0,
                    });
                    path.line_to(Point {
                        x: cx + tri_w,
                        y: 0.0,
                    });
                    path.line_to(Point { x: cx, y: tri_w });
                    path.line_to(Point {
                        x: cx - tri_w,
                        y: 0.0,
                    });
                });
                frame.fill(&tri, Fill::Color(Color::WHITE));

                let tri = Path::new(|path| {
                    path.move_to(Point {
                        x: cx - tri_w,
                        y: box_size.height,
                    });
                    path.line_to(Point {
                        x: cx + tri_w,
                        y: box_size.height,
                    });
                    path.line_to(Point {
                        x: cx,
                        y: box_size.height - tri_w,
                    });
                    path.line_to(Point {
                        x: cx - tri_w,
                        y: box_size.height,
                    });
                });
                frame.fill(&tri, Fill::Color(Color::WHITE));
            }

            frame.fill_text(canvas::Text {
                content: color_hex_str(&self.theme[i]),
                position: Point {
                    x: anchor.x + box_size.width / 2.0,
                    y: box_size.height,
                },
                ..text
            });
        }

        text.vertical_alignment = VerticalAlignment::Bottom;

        let hsl = Hsl::from(Srgb::from(self.color));
        for i in 0..self.theme.len() {
            let pct = (i as f32 + 1.0) / (self.theme.len() as f32 + 1.0);
            let graded = Hsl {
                lightness: 1.0 - pct,
                ..hsl
            };
            let color: Color = Srgb::from(graded.clamp()).into();

            let anchor = Point {
                x: (i as f32) * box_size.width,
                y: box_size.height + 2.0 * pad,
            };
            let rect = Path::new(|path| {
                path.rectangle(anchor, box_size);
            });
            frame.fill(&rect, Fill::Color(color));

            frame.fill_text(canvas::Text {
                content: color_hex_str(&color),
                position: Point {
                    x: anchor.x + box_size.width / 2.0,
                    y: box_size.height + 2.0 * pad,
                },
                ..text
            });
        }
    }
}

fn color_hex_str(color: &Color) -> String {
    format!(
        "#{:x}{:x}{:x}",
        (255.0 * color.r).round() as u8,
        (255.0 * color.g).round() as u8,
        (255.0 * color.b).round() as u8
    )
}
