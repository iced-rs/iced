use crate::core::alignment;
use crate::core::text::{LineHeight, Shaping};
use crate::core::{Color, Font, Pixels, Point, Size, Vector};
use crate::geometry::Path;
use crate::text;

/// A bunch of text that can be drawn to a canvas
#[derive(Debug, Clone)]
pub struct Text {
    /// The contents of the text
    pub content: String,
    /// The position of the text relative to the alignment properties.
    /// By default, this position will be relative to the top-left corner coordinate meaning that
    /// if the horizontal and vertical alignments are unchanged, this property will tell where the
    /// top-left corner of the text should be placed.
    /// By changing the horizontal_alignment and vertical_alignment properties, you are are able to
    /// change what part of text is placed at this positions.
    /// For example, when the horizontal_alignment and vertical_alignment are set to Center, the
    /// center of the text will be placed at the given position NOT the top-left coordinate.
    pub position: Point,
    /// The color of the text
    pub color: Color,
    /// The size of the text
    pub size: Pixels,
    /// The line height of the text.
    pub line_height: LineHeight,
    /// The font of the text
    pub font: Font,
    /// The horizontal alignment of the text
    pub horizontal_alignment: alignment::Horizontal,
    /// The vertical alignment of the text
    pub vertical_alignment: alignment::Vertical,
    /// The shaping strategy of the text.
    pub shaping: Shaping,
}

impl Text {
    /// Computes the [`Path`]s of the [`Text`] and draws them using
    /// the given closure.
    pub fn draw_with(&self, mut f: impl FnMut(Path, Color)) {
        let mut font_system =
            text::font_system().write().expect("Write font system");

        let mut buffer = cosmic_text::BufferLine::new(
            &self.content,
            cosmic_text::AttrsList::new(text::to_attributes(self.font)),
            text::to_shaping(self.shaping),
        );

        let layout = buffer.layout(
            font_system.raw(),
            self.size.0,
            f32::MAX,
            cosmic_text::Wrap::None,
        );

        let translation_x = match self.horizontal_alignment {
            alignment::Horizontal::Left => self.position.x,
            alignment::Horizontal::Center | alignment::Horizontal::Right => {
                let mut line_width = 0.0f32;

                for line in layout.iter() {
                    line_width = line_width.max(line.w);
                }

                if self.horizontal_alignment == alignment::Horizontal::Center {
                    self.position.x - line_width / 2.0
                } else {
                    self.position.x - line_width
                }
            }
        };

        let translation_y = {
            let line_height = self.line_height.to_absolute(self.size);

            match self.vertical_alignment {
                alignment::Vertical::Top => self.position.y,
                alignment::Vertical::Center => {
                    self.position.y - line_height.0 / 2.0
                }
                alignment::Vertical::Bottom => self.position.y - line_height.0,
            }
        };

        let mut swash_cache = cosmic_text::SwashCache::new();

        for run in layout.iter() {
            for glyph in run.glyphs.iter() {
                let physical_glyph = glyph.physical((0.0, 0.0), 1.0);

                let start_x = translation_x + glyph.x + glyph.x_offset;
                let start_y = translation_y + glyph.y_offset + self.size.0;
                let offset = Vector::new(start_x, start_y);

                if let Some(commands) = swash_cache.get_outline_commands(
                    font_system.raw(),
                    physical_glyph.cache_key,
                ) {
                    let glyph = Path::new(|path| {
                        use cosmic_text::Command;

                        for command in commands {
                            match command {
                                Command::MoveTo(p) => {
                                    path.move_to(
                                        Point::new(p.x, -p.y) + offset,
                                    );
                                }
                                Command::LineTo(p) => {
                                    path.line_to(
                                        Point::new(p.x, -p.y) + offset,
                                    );
                                }
                                Command::CurveTo(control_a, control_b, to) => {
                                    path.bezier_curve_to(
                                        Point::new(control_a.x, -control_a.y)
                                            + offset,
                                        Point::new(control_b.x, -control_b.y)
                                            + offset,
                                        Point::new(to.x, -to.y) + offset,
                                    );
                                }
                                Command::QuadTo(control, to) => {
                                    path.quadratic_curve_to(
                                        Point::new(control.x, -control.y)
                                            + offset,
                                        Point::new(to.x, -to.y) + offset,
                                    );
                                }
                                Command::Close => {
                                    path.close();
                                }
                            }
                        }
                    });

                    f(glyph, self.color);
                } else {
                    // TODO: Raster image support for `Canvas`
                    let [r, g, b, a] = self.color.into_rgba8();

                    swash_cache.with_pixels(
                        font_system.raw(),
                        physical_glyph.cache_key,
                        cosmic_text::Color::rgba(r, g, b, a),
                        |x, y, color| {
                            f(
                                Path::rectangle(
                                    Point::new(x as f32, y as f32) + offset,
                                    Size::new(1.0, 1.0),
                                ),
                                Color::from_rgba8(
                                    color.r(),
                                    color.g(),
                                    color.b(),
                                    color.a() as f32 / 255.0,
                                ),
                            );
                        },
                    );
                }
            }
        }
    }
}

impl Default for Text {
    fn default() -> Text {
        Text {
            content: String::new(),
            position: Point::ORIGIN,
            color: Color::BLACK,
            size: Pixels(16.0),
            line_height: LineHeight::Relative(1.2),
            font: Font::default(),
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            shaping: Shaping::Basic,
        }
    }
}

impl From<String> for Text {
    fn from(content: String) -> Text {
        Text {
            content,
            ..Default::default()
        }
    }
}

impl From<&str> for Text {
    fn from(content: &str) -> Text {
        String::from(content).into()
    }
}
