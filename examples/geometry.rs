//! This example showcases a simple native custom widget that renders using
//! arbitrary low-level geometry.
mod rainbow {
    // For now, to implement a custom native widget you will need to add
    // `iced_native` and `iced_wgpu` to your dependencies.
    //
    // Then, you simply need to define your widget type and implement the
    // `iced_native::Widget` trait with the `iced_wgpu::Renderer`.
    //
    // Of course, you can choose to make the implementation renderer-agnostic,
    // if you wish to, by creating your own `Renderer` trait, which could be
    // implemented by `iced_wgpu` and other renderers.
    use iced_native::{
        layout,  Element, Geometry2D, Hasher, Layout, Length,
        MouseCursor, Point, Size, Vertex2D, Widget,
    };
    use iced_wgpu::{Primitive, Renderer};

    pub struct Rainbow {
        dimen: u16,
    }

    impl Rainbow {
        pub fn new(dimen: u16) -> Self {
            Self { dimen }
        }
    }

    impl<Message> Widget<Message, Renderer> for Rainbow {
        fn width(&self) -> Length {
            Length::Shrink
        }

        fn height(&self) -> Length {
            Length::Shrink
        }

        fn layout(
            &self,
            _renderer: &Renderer,
            _limits: &layout::Limits,
        ) -> layout::Node {
            layout::Node::new(Size::new(
                f32::from(self.dimen),
                f32::from(self.dimen),
            ))
        }

        fn hash_layout(&self, state: &mut Hasher) {
            use std::hash::Hash;

            self.dimen.hash(state);
        }

        fn draw(
            &self,
            _renderer: &mut Renderer,
            layout: Layout<'_>,
            cursor_position: Point,
        ) -> (Primitive, MouseCursor) {
            let b = layout.bounds();

            // R O Y G B I V
            let color_r = [1.0, 0.0, 0.0, 1.0];
            let color_o = [1.0, 0.5, 0.0, 1.0];
            let color_y = [1.0, 1.0, 0.0, 1.0];
            let color_g = [0.0, 1.0, 0.0, 1.0];
            let color_gb = [0.0, 1.0, 0.5, 1.0];
            let color_b = [0.0, 0.2, 1.0, 1.0];
            let color_i = [0.5, 0.0, 1.0, 1.0];
            let color_v = [0.75, 0.0, 0.5, 1.0];

            let posn_center = {
                if b.contains(cursor_position) {
                    [cursor_position.x, cursor_position.y]
                } else {
                    [b.x + (b.width / 2.0), b.y + (b.height / 2.0)]
                }
            };

            let posn_tl = [b.x, b.y];
            let posn_t = [b.x + (b.width / 2.0), b.y];
            let posn_tr = [b.x + b.width, b.y];
            let posn_r = [b.x + b.width, b.y + (b.height / 2.0)];
            let posn_br = [b.x + b.width, b.y + b.height];
            let posn_b = [b.x + (b.width / 2.0), b.y + b.height];
            let posn_bl = [b.x, b.y + b.height];
            let posn_l = [b.x, b.y + (b.height / 2.0)];

            (
                Primitive::Geometry2D {
                    geometry: Geometry2D {
                        vertices: vec![
                            Vertex2D {
                                position: posn_center,
                                color: [1.0, 1.0, 1.0, 1.0],
                            },
                            Vertex2D {
                                position: posn_tl,
                                color: color_r,
                            },
                            Vertex2D {
                                position: posn_t,
                                color: color_o,
                            },
                            Vertex2D {
                                position: posn_tr,
                                color: color_y,
                            },
                            Vertex2D {
                                position: posn_r,
                                color: color_g,
                            },
                            Vertex2D {
                                position: posn_br,
                                color: color_gb,
                            },
                            Vertex2D {
                                position: posn_b,
                                color: color_b,
                            },
                            Vertex2D {
                                position: posn_bl,
                                color: color_i,
                            },
                            Vertex2D {
                                position: posn_l,
                                color: color_v,
                            },
                        ],
                        indices: vec![
                            0, 1, 2, // TL
                            0, 2, 3, // T
                            0, 3, 4, // TR
                            0, 4, 5, // R
                            0, 5, 6, // BR
                            0, 6, 7, // B
                            0, 7, 8, // BL
                            0, 8, 1, // L
                        ],
                    },
                },
                MouseCursor::OutOfBounds,
            )
        }
    }

    impl<'a, Message> Into<Element<'a, Message, Renderer>> for Rainbow {
        fn into(self) -> Element<'a, Message, Renderer> {
            Element::new(self)
        }
    }
}

use iced::{
    scrollable, settings::Window,  Align, Column, Container, Element,
    Length, Sandbox, Scrollable, Settings,  Text,
};
use rainbow::Rainbow;

pub fn main() {
    Example::run(Settings {
        window: Window {
            size: (660, 660),
            resizable: true,
            decorations: true,
        },
    })
}

struct Example {
    dimen: u16,
    scroll: scrollable::State,
}

#[derive(Debug, Clone, Copy)]
enum Message {}

impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        Example {
            dimen: 500,
            scroll: scrollable::State::new(),
        }
    }

    fn title(&self) -> String {
        String::from("Custom 2D geometry - Iced")
    }

    fn update(&mut self, _: Message) {}

    fn view(&mut self) -> Element<Message> {
        let content = Column::new()
            .padding(20)
            .spacing(20)
            .max_width(500)
            .align_items(Align::Start)
            .push(Rainbow::new(self.dimen))
            .push(
                Text::new(String::from("In this example we draw a custom widget Rainbow, using the \
                Geometry2D primitive. This primitive supplies a list of triangles, expressed as vertices and indices."))
                    .width(Length::Shrink),
            )
            .push(
                Text::new(String::from("Move your cursor over it, and see the center vertex follow you!"))
                    .width(Length::Shrink),
            )
            .push(
                Text::new(String::from("Every Vertex2D defines its own color. You could use the \
                Geometry2D primitive to render virtually any two-dimensional geometry for your widget."))
                    .width(Length::Shrink),
            );

        let scrollable =
            Scrollable::new(&mut self.scroll).push(Container::new(content));

        Container::new(scrollable)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
