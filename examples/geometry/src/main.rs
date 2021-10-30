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
    use iced_graphics::renderer::{self, Renderer};
    use iced_graphics::{Backend, Primitive};

    use iced_native::{
        layout, Element, Hasher, Layout, Length, Point, Rectangle, Size,
        Vector, Widget,
    };

    pub struct Rainbow;

    impl Rainbow {
        pub fn new() -> Self {
            Self
        }
    }

    impl<Message, B> Widget<Message, Renderer<B>> for Rainbow
    where
        B: Backend,
    {
        fn width(&self) -> Length {
            Length::Fill
        }

        fn height(&self) -> Length {
            Length::Shrink
        }

        fn layout(
            &self,
            _renderer: &Renderer<B>,
            limits: &layout::Limits,
        ) -> layout::Node {
            let size = limits.width(Length::Fill).resolve(Size::ZERO);

            layout::Node::new(Size::new(size.width, size.width))
        }

        fn hash_layout(&self, _state: &mut Hasher) {}

        fn draw(
            &self,
            renderer: &mut Renderer<B>,
            _style: &renderer::Style,
            layout: Layout<'_>,
            cursor_position: Point,
            _viewport: &Rectangle,
        ) {
            use iced_graphics::triangle::{Mesh2D, Vertex2D};
            use iced_native::Renderer as _;

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
                    [cursor_position.x - b.x, cursor_position.y - b.y]
                } else {
                    [b.width / 2.0, b.height / 2.0]
                }
            };

            let posn_tl = [0.0, 0.0];
            let posn_t = [b.width / 2.0, 0.0];
            let posn_tr = [b.width, 0.0];
            let posn_r = [b.width, b.height / 2.0];
            let posn_br = [b.width, b.height];
            let posn_b = [(b.width / 2.0), b.height];
            let posn_bl = [0.0, b.height];
            let posn_l = [0.0, b.height / 2.0];

            let mesh = Primitive::Mesh2D {
                size: b.size(),
                buffers: Mesh2D {
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
            };

            renderer.with_translation(Vector::new(b.x, b.y), |renderer| {
                renderer.draw_primitive(mesh);
            });
        }
    }

    impl<'a, Message, B> Into<Element<'a, Message, Renderer<B>>> for Rainbow
    where
        B: Backend,
    {
        fn into(self) -> Element<'a, Message, Renderer<B>> {
            Element::new(self)
        }
    }
}

use iced::{
    scrollable, Alignment, Column, Container, Element, Length, Sandbox,
    Scrollable, Settings, Text,
};
use rainbow::Rainbow;

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

struct Example {
    scroll: scrollable::State,
}

impl Sandbox for Example {
    type Message = ();

    fn new() -> Self {
        Example {
            scroll: scrollable::State::new(),
        }
    }

    fn title(&self) -> String {
        String::from("Custom 2D geometry - Iced")
    }

    fn update(&mut self, _: ()) {}

    fn view(&mut self) -> Element<()> {
        let content = Column::new()
            .padding(20)
            .spacing(20)
            .max_width(500)
            .align_items(Alignment::Start)
            .push(Rainbow::new())
            .push(Text::new(
                "In this example we draw a custom widget Rainbow, using \
                 the Mesh2D primitive. This primitive supplies a list of \
                 triangles, expressed as vertices and indices.",
            ))
            .push(Text::new(
                "Move your cursor over it, and see the center vertex \
                 follow you!",
            ))
            .push(Text::new(
                "Every Vertex2D defines its own color. You could use the \
                 Mesh2D primitive to render virtually any two-dimensional \
                 geometry for your widget.",
            ));

        let scrollable = Scrollable::new(&mut self.scroll)
            .push(Container::new(content).width(Length::Fill).center_x());

        Container::new(scrollable)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_y()
            .into()
    }
}
