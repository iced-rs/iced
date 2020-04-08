use iced::{
    canvas, slider, Canvas, Color, Column, Element, Length, Point, Row,
    Sandbox, Settings, Slider, Text,
};
use iced_core::palette::{self, Limited};

pub fn main() {
    ColorPalette::run(Settings::default())
}

#[derive(Debug, Default)]
pub struct State {
    color: Color,
    theme: Vec<Color>,
}

fn generate_theme(base_color: &Color) -> Vec<Color> {
    use palette::{Hsl, Hue, Shade, Srgb};
    let mut theme = Vec::<Color>::new();
    // Convert to linear color for manipulation
    let srgb = Srgb::from(*base_color);

    let hsl = Hsl::from(srgb);

    theme.push(Srgb::from(hsl.shift_hue(-120.0)).clamp().into());
    theme.push(Srgb::from(hsl.shift_hue(-115.0).darken(0.075)).clamp().into());
    theme.push(Srgb::from(hsl.darken(0.075)).clamp().into());
    theme.push(*base_color);
    theme.push(Srgb::from(hsl.lighten(0.075)).clamp().into());
    theme.push(Srgb::from(hsl.shift_hue(115.0).darken(0.075)).clamp().into());
    theme.push(Srgb::from(hsl.shift_hue(120.0)).clamp().into());
    theme
}

pub struct ColorPalette {
    state: State,
    rgb_sliders: [slider::State; 3],
    hsl_sliders: [slider::State; 3],
    hsv_sliders: [slider::State; 3],
    hwb_sliders: [slider::State; 3],
    lab_sliders: [slider::State; 3],
    lch_sliders: [slider::State; 3],
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
            rgb_sliders: triple_slider(),
            hsl_sliders: triple_slider(),
            hsv_sliders: triple_slider(),
            hwb_sliders: triple_slider(),
            lab_sliders: triple_slider(),
            lch_sliders: triple_slider(),
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
        let [rgb1, rgb2, rgb3] = &mut self.rgb_sliders;
        let [hsl1, hsl2, hsl3] = &mut self.hsl_sliders;
        let [hsv1, hsv2, hsv3] = &mut self.hsv_sliders;
        let [hwb1, hwb2, hwb3] = &mut self.hwb_sliders;
        let [lab1, lab2, lab3] = &mut self.lab_sliders;
        let [lch1, lch2, lch3] = &mut self.lch_sliders;

        let color = self.state.color;
        let srgb = palette::Srgb::from(self.state.color);
        let hsl = palette::Hsl::from(srgb);
        let hsv = palette::Hsv::from(srgb);
        let hwb = palette::Hwb::from(srgb);
        let lab = palette::Lab::from(srgb);
        let lch = palette::Lch::from(srgb);

        Column::new()
            .padding(20)
            .spacing(20)
            .push(
                Row::new()
                    .spacing(10)
                    .push(Text::new("RGB"))
                    .push(Slider::new(rgb1, 0.0..=1.0, color.r, move |r| {
                        Message::RgbColorChanged(Color { r, ..color })
                    }))
                    .push(Slider::new(rgb2, 0.0..=1.0, color.g, move |g| {
                        Message::RgbColorChanged(Color { g, ..color })
                    }))
                    .push(Slider::new(rgb3, 0.0..=1.0, color.b, move |b| {
                        Message::RgbColorChanged(Color { b, ..color })
                    })),
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(Text::new("HSL"))
                    .push(Slider::new(
                        hsl1,
                        0.0..=360.0,
                        hsl.hue.to_positive_degrees(),
                        move |hue| {
                            Message::HslColorChanged(palette::Hsl {
                                hue: palette::RgbHue::from_degrees(hue),
                                ..hsl
                            })
                        },
                    ))
                    .push(Slider::new(
                        hsl2,
                        0.0..=1.0,
                        hsl.saturation,
                        move |saturation| {
                            Message::HslColorChanged(palette::Hsl {
                                saturation,
                                ..hsl
                            })
                        },
                    ))
                    .push(Slider::new(
                        hsl3,
                        0.0..=1.0,
                        hsl.lightness,
                        move |lightness| {
                            Message::HslColorChanged(palette::Hsl {
                                lightness,
                                ..hsl
                            })
                        },
                    )),
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(Text::new("HSV"))
                    .push(Slider::new(
                        hsv1,
                        0.0..=360.0,
                        hsv.hue.to_positive_degrees(),
                        move |hue| {
                            Message::HsvColorChanged(palette::Hsv {
                                hue: palette::RgbHue::from_degrees(hue),
                                ..hsv
                            })
                        },
                    ))
                    .push(Slider::new(
                        hsv2,
                        0.0..=1.0,
                        hsv.saturation,
                        move |saturation| {
                            Message::HsvColorChanged(palette::Hsv {
                                saturation,
                                ..hsv
                            })
                        },
                    ))
                    .push(Slider::new(
                        hsv3,
                        0.0..=1.0,
                        hsv.value,
                        move |value| {
                            Message::HsvColorChanged(palette::Hsv {
                                value,
                                ..hsv
                            })
                        },
                    )),
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(Text::new("HWB"))
                    .push(Slider::new(
                        hwb1,
                        0.0..=360.0,
                        hwb.hue.to_positive_degrees(),
                        move |hue| {
                            Message::HwbColorChanged(palette::Hwb {
                                hue: palette::RgbHue::from_degrees(hue),
                                ..hwb
                            })
                        },
                    ))
                    .push(Slider::new(
                        hwb2,
                        0.0..=1.0,
                        hwb.whiteness,
                        move |whiteness| {
                            Message::HwbColorChanged(palette::Hwb {
                                whiteness,
                                ..hwb
                            })
                        },
                    ))
                    .push(Slider::new(
                        hwb3,
                        0.0..=1.0,
                        hwb.blackness,
                        move |blackness| {
                            Message::HwbColorChanged(palette::Hwb {
                                blackness,
                                ..hwb
                            })
                        },
                    )),
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(Text::new("Lab"))
                    .push(Slider::new(lab1, 0.0..=100.0, lab.l, move |l| {
                        Message::LabColorChanged(palette::Lab { l, ..lab })
                    }))
                    .push(Slider::new(lab2, -128.0..=127.0, lab.a, move |a| {
                        Message::LabColorChanged(palette::Lab { a, ..lab })
                    }))
                    .push(Slider::new(lab3, -128.0..=127.0, lab.b, move |b| {
                        Message::LabColorChanged(palette::Lab { b, ..lab })
                    })),
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(Text::new("Lch"))
                    .push(Slider::new(lch1, 0.0..=100.0, lch.l, move |l| {
                        Message::LchColorChanged(palette::Lch { l, ..lch })
                    }))
                    .push(Slider::new(
                        lch2,
                        0.0..=128.0,
                        lch.chroma,
                        move |chroma| {
                            Message::LchColorChanged(palette::Lch {
                                chroma,
                                ..lch
                            })
                        },
                    ))
                    .push(Slider::new(
                        lch3,
                        0.0..=360.0,
                        lch.hue.to_positive_degrees(),
                        move |hue| {
                            Message::LchColorChanged(palette::Lch {
                                hue: palette::LabHue::from_degrees(hue),
                                ..lch
                            })
                        },
                    )),
            )
            .push(
                Canvas::new()
                    .width(Length::Fill)
                    .height(Length::Units(150))
                    .push(self.canvas_layer.with(&self.state)),
            )
            .into()
    }
}

impl State {
    pub fn new() -> State {
        let base = Color::from_rgb8(27, 135, 199);
        State {
            color: base,
            theme: generate_theme(&base),
        }
    }
}

impl canvas::Drawable for State {
    fn draw(&self, frame: &mut canvas::Frame) {
        use canvas::{Fill, Path};
        if self.theme.len() == 0 {
            println!("Zero len");
            return;
        }

        let box_width = frame.width() / self.theme.len() as f32;
        for i in 0..self.theme.len() {
            let anchor = Point {
                x: (i as f32) * box_width,
                y: 0.0,
            };
            let rect = Path::new(|path| {
                path.move_to(anchor);
                path.line_to(Point { x: anchor.x + box_width, y: anchor.y });
                path.line_to(Point {
                    x: anchor.x + box_width,
                    y: anchor.y + frame.height(),
                });
                path.line_to(Point { x: anchor.x, y: anchor.y + frame.height() });
            });
            frame.fill(&rect, Fill::Color(self.theme[i]));
        }
    }
}
