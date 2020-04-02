use iced::{
    slider, Color, Column, Element, Row, Sandbox, Settings, Slider, Text,
};
use iced_core::palette::{self, Limited};

pub fn main() {
    ColorPalette::run(Settings::default())
}

#[derive(Default)]
pub struct ColorPalette {
    base_color: Color,
    rgb_sliders: [slider::State; 3],
    hsl_sliders: [slider::State; 3],
    hsv_sliders: [slider::State; 3],
    hwb_sliders: [slider::State; 3],
    lab_sliders: [slider::State; 3],
    lch_sliders: [slider::State; 3],
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
        let mut s = Self::default();
        s.base_color = Color::from_rgb8(27, 135, 199);
        s
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
        self.base_color = Color::from(srgb);
    }

    fn view(&mut self) -> Element<Message> {
        let [rgb1, rgb2, rgb3] = &mut self.rgb_sliders;
        let [hsl1, hsl2, hsl3] = &mut self.hsl_sliders;
        let [hsv1, hsv2, hsv3] = &mut self.hsv_sliders;
        let [hwb1, hwb2, hwb3] = &mut self.hwb_sliders;
        let [lab1, lab2, lab3] = &mut self.lab_sliders;
        let [lch1, lch2, lch3] = &mut self.lch_sliders;

        let color = self.base_color;
        let srgb = palette::Srgb::from(self.base_color);
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
            .into()
    }
}
