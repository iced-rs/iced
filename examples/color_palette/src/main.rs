use iced::{
    canvas, slider, Canvas, Color, Column, Element, Length, Point, Row,
    Sandbox, Settings, Slider, Text,
};
use palette::{self, Limited};

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

pub struct ColorPalette {
    state: State,
    rgb_sliders: [slider::State; 3],
    hsl_sliders: [slider::State; 3],
    hsv_sliders: [slider::State; 3],
    hwb_sliders: [slider::State; 3],
    lab_sliders: [slider::State; 3],
    lch_sliders: [slider::State; 3],
    rgb_text_value: String,
    hsl_text_value: String,
    hsv_text_value: String,
    hwb_text_value: String,
    lab_text_value: String,
    lch_text_value: String,
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

        let state = State::new();
        let rgb_text_value = color_str(&state.color, ColorFormat::Rgb);
        let hsl_text_value = color_str(&state.color, ColorFormat::Hsl);
        let hsv_text_value = color_str(&state.color, ColorFormat::Hsv);
        let hwb_text_value = color_str(&state.color, ColorFormat::Hwb);
        let lab_text_value = color_str(&state.color, ColorFormat::Lab);
        let lch_text_value = color_str(&state.color, ColorFormat::Lch);

        ColorPalette {
            state,
            rgb_sliders: triple_slider(),
            hsl_sliders: triple_slider(),
            hsv_sliders: triple_slider(),
            hwb_sliders: triple_slider(),
            lab_sliders: triple_slider(),
            lch_sliders: triple_slider(),
            rgb_text_value,
            hsl_text_value,
            hsv_text_value,
            hwb_text_value,
            lab_text_value,
            lch_text_value,
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

        // Set text
        self.rgb_text_value = color_str(&self.state.color, ColorFormat::Rgb);
        self.hsl_text_value = color_str(&self.state.color, ColorFormat::Hsl);
        self.hsv_text_value = color_str(&self.state.color, ColorFormat::Hsv);
        self.hwb_text_value = color_str(&self.state.color, ColorFormat::Hwb);
        self.lab_text_value = color_str(&self.state.color, ColorFormat::Lab);
        self.lch_text_value = color_str(&self.state.color, ColorFormat::Lch);
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
            .padding(10)
            .spacing(10)
            .push(
                Row::new()
                    .spacing(10)
                    .push(Text::new("RGB").width(Length::Units(50)))
                    .push(Slider::new(rgb1, 0.0..=1.0, color.r, move |r| {
                        Message::RgbColorChanged(Color { r, ..color })
                    }))
                    .push(Slider::new(rgb2, 0.0..=1.0, color.g, move |g| {
                        Message::RgbColorChanged(Color { g, ..color })
                    }))
                    .push(Slider::new(rgb3, 0.0..=1.0, color.b, move |b| {
                        Message::RgbColorChanged(Color { b, ..color })
                    }))
                    .push(
                        Text::new(&self.rgb_text_value)
                            .width(Length::Units(185))
                            .size(16),
                    ),
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(Text::new("HSL").width(Length::Units(50)))
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
                    ))
                    .push(
                        Text::new(&self.hsl_text_value)
                            .width(Length::Units(185))
                            .size(16),
                    ),
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(Text::new("HSV").width(Length::Units(50)))
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
                    ))
                    .push(
                        Text::new(&self.hsv_text_value)
                            .width(Length::Units(185))
                            .size(16),
                    ),
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(Text::new("HWB").width(Length::Units(50)))
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
                    ))
                    .push(
                        Text::new(&self.hwb_text_value)
                            .width(Length::Units(185))
                            .size(16),
                    ),
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(Text::new("Lab").width(Length::Units(50)))
                    .push(Slider::new(lab1, 0.0..=100.0, lab.l, move |l| {
                        Message::LabColorChanged(palette::Lab { l, ..lab })
                    }))
                    .push(Slider::new(lab2, -128.0..=127.0, lab.a, move |a| {
                        Message::LabColorChanged(palette::Lab { a, ..lab })
                    }))
                    .push(Slider::new(lab3, -128.0..=127.0, lab.b, move |b| {
                        Message::LabColorChanged(palette::Lab { b, ..lab })
                    }))
                    .push(
                        Text::new(&self.lab_text_value)
                            .width(Length::Units(185))
                            .size(16),
                    ),
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(Text::new("Lch").width(Length::Units(50)))
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
                    ))
                    .push(
                        Text::new(&self.lch_text_value)
                            .width(Length::Units(185))
                            .size(16),
                    ),
            )
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
        use palette::{Hsl, Srgb};

        if self.theme.len() == 0 {
            println!("Zero len");
            return;
        }

        let pad = 20.0;

        let box_width = frame.width() / self.theme.len() as f32;
        let box_height = frame.height() / 2.0 - pad;

        let mut text = canvas::Text::default();
        text.horizontal_alignment = HorizontalAlignment::Center;
        text.vertical_alignment = VerticalAlignment::Top;
        text.size = 15.0;

        for i in 0..self.theme.len() {
            let anchor = Point {
                x: (i as f32) * box_width,
                y: 0.0,
            };
            let rect = Path::new(|path| {
                path.move_to(anchor);
                path.line_to(Point {
                    x: anchor.x + box_width,
                    y: anchor.y,
                });
                path.line_to(Point {
                    x: anchor.x + box_width,
                    y: anchor.y + box_height,
                });
                path.line_to(Point {
                    x: anchor.x,
                    y: anchor.y + box_height,
                });
            });
            frame.fill(&rect, Fill::Color(self.theme[i]));

            if self.theme[i] == self.color {
                let cx = anchor.x + box_width / 2.0;
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
                        y: box_height,
                    });
                    path.line_to(Point {
                        x: cx + tri_w,
                        y: box_height,
                    });
                    path.line_to(Point {
                        x: cx,
                        y: box_height - tri_w,
                    });
                    path.line_to(Point {
                        x: cx - tri_w,
                        y: box_height,
                    });
                });
                frame.fill(&tri, Fill::Color(Color::WHITE));
            }

            frame.fill_text(canvas::Text {
                content: color_str(&self.theme[i], ColorFormat::Hex),
                position: Point {
                    x: anchor.x + box_width / 2.0,
                    y: box_height,
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
                x: (i as f32) * box_width,
                y: box_height + 2.0 * pad,
            };
            let rect = Path::new(|path| {
                path.move_to(anchor);
                path.line_to(Point {
                    x: anchor.x + box_width,
                    y: anchor.y,
                });
                path.line_to(Point {
                    x: anchor.x + box_width,
                    y: anchor.y + box_height,
                });
                path.line_to(Point {
                    x: anchor.x,
                    y: anchor.y + box_height,
                });
            });
            frame.fill(&rect, Fill::Color(color));

            frame.fill_text(canvas::Text {
                content: color_str(&color, ColorFormat::Hex),
                position: Point {
                    x: anchor.x + box_width / 2.0,
                    y: box_height + 2.0 * pad,
                },
                ..text
            });
        }
    }
}

enum ColorFormat {
    Hex,
    Rgb,
    Hsl,
    Hsv,
    Hwb,
    Lab,
    Lch,
}

fn color_str(color: &Color, color_format: ColorFormat) -> String {
    let srgb = palette::Srgb::from(*color);
    let hsl = palette::Hsl::from(srgb);
    let hsv = palette::Hsv::from(srgb);
    let hwb = palette::Hwb::from(srgb);
    let lab = palette::Lab::from(srgb);
    let lch = palette::Lch::from(srgb);

    match color_format {
        ColorFormat::Hex => format!(
            "#{:x}{:x}{:x}",
            (255.0 * color.r).round() as u8,
            (255.0 * color.g).round() as u8,
            (255.0 * color.b).round() as u8
        ),
        ColorFormat::Rgb => format!(
            "rgb({:.0}, {:.0}, {:.0})",
            255.0 * color.r,
            255.0 * color.g,
            255.0 * color.b
        ),
        ColorFormat::Hsl => format!(
            "hsl({:.1}, {:.1}%, {:.1}%)",
            hsl.hue.to_positive_degrees(),
            100.0 * hsl.saturation,
            100.0 * hsl.lightness
        ),
        ColorFormat::Hsv => format!(
            "hsv({:.1}, {:.1}%, {:.1}%)",
            hsv.hue.to_positive_degrees(),
            100.0 * hsv.saturation,
            100.0 * hsv.value
        ),
        ColorFormat::Hwb => format!(
            "hwb({:.1}, {:.1}%, {:.1}%)",
            hwb.hue.to_positive_degrees(),
            100.0 * hwb.whiteness,
            100.0 * hwb.blackness
        ),
        ColorFormat::Lab => {
            format!("Lab({:.1}, {:.1}, {:.1})", lab.l, lab.a, lab.b)
        }
        ColorFormat::Lch => format!(
            "Lch({:.1}, {:.1}, {:.1})",
            lch.l,
            lch.chroma,
            lch.hue.to_positive_degrees()
        ),
    }
}
