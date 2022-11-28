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
    use iced_graphics::triangle::ColoredVertex2D;
    use iced_graphics::{Backend, Primitive};

    use iced_native::layout;
    use iced_native::widget::{self, Widget};
    use iced_native::{
        Element, Layout, Length, Point, Rectangle, Size, Vector,
    };

    #[derive(Debug, Clone, Copy, Default)]
    pub struct Rainbow;

    pub fn rainbow() -> Rainbow {
        Rainbow
    }

    impl<Message, B, T> Widget<Message, Renderer<B, T>> for Rainbow
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
            _renderer: &Renderer<B, T>,
            limits: &layout::Limits,
        ) -> layout::Node {
            let size = limits.width(Length::Fill).resolve(Size::ZERO);

            layout::Node::new(Size::new(size.width, size.width))
        }

        fn draw(
            &self,
            _tree: &widget::Tree,
            renderer: &mut Renderer<B, T>,
            _theme: &T,
            _style: &renderer::Style,
            layout: Layout<'_>,
            cursor_position: Point,
            _viewport: &Rectangle,
        ) {
            use iced_graphics::triangle::Mesh2D;
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

            let mesh = Primitive::SolidMesh {
                size: b.size(),
                buffers: Mesh2D {
                    vertices: vec![
                        ColoredVertex2D {
                            position: posn_center,
                            color: [1.0, 1.0, 1.0, 1.0],
                        },
                        ColoredVertex2D {
                            position: posn_tl,
                            color: color_r,
                        },
                        ColoredVertex2D {
                            position: posn_t,
                            color: color_o,
                        },
                        ColoredVertex2D {
                            position: posn_tr,
                            color: color_y,
                        },
                        ColoredVertex2D {
                            position: posn_r,
                            color: color_g,
                        },
                        ColoredVertex2D {
                            position: posn_br,
                            color: color_gb,
                        },
                        ColoredVertex2D {
                            position: posn_b,
                            color: color_b,
                        },
                        ColoredVertex2D {
                            position: posn_bl,
                            color: color_i,
                        },
                        ColoredVertex2D {
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

    impl<'a, Message, B, T> From<Rainbow> for Element<'a, Message, Renderer<B, T>>
    where
        B: Backend,
    {
        fn from(rainbow: Rainbow) -> Self {
            Self::new(rainbow)
        }
    }
}

use iced::widget::{column, container, scrollable};
use iced::{Element, Length, Sandbox, Settings};
use rainbow::rainbow;

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

struct Example;

impl Sandbox for Example {
    type Message = ();

    fn new() -> Self {
        Self
    }

    fn title(&self) -> String {
        String::from("Custom 2D geometry - Iced")
    }

    fn update(&mut self, _: ()) {}

    fn view(&self) -> Element<()> {
        let content = column![
            rainbow(),
            "In this example we draw a custom widget Rainbow, using \
                 the Mesh2D primitive. This primitive supplies a list of \
                 triangles, expressed as vertices and indices.",
            "Move your cursor over it, and see the center vertex \
                 follow you!",
            "Every Vertex2D defines its own color. You could use the \
                 Mesh2D primitive to render virtually any two-dimensional \
                 geometry for your widget.",
        ]
        .padding(20)
        .spacing(20)
        .max_width(500);

        let scrollable =
            scrollable(container(content).width(Length::Fill).center_x());

        container(scrollable)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_y()
            .into()
    }
}
