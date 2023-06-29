//! This example showcases a simple native custom widget that renders using
//! arbitrary low-level geometry.
mod rainbow {
    use iced::advanced::graphics::color;
    use iced::advanced::layout::{self, Layout};
    use iced::advanced::renderer;
    use iced::advanced::widget::{self, Widget};
    use iced::mouse;
    use iced::{Element, Length, Rectangle, Renderer, Size, Theme, Vector};

    #[derive(Debug, Clone, Copy, Default)]
    pub struct Rainbow;

    pub fn rainbow() -> Rainbow {
        Rainbow
    }

    impl<Message> Widget<Message, Renderer> for Rainbow {
        fn width(&self) -> Length {
            Length::Fill
        }

        fn height(&self) -> Length {
            Length::Shrink
        }

        fn layout(
            &self,
            _renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            let size = limits.width(Length::Fill).resolve(Size::ZERO);

            layout::Node::new(Size::new(size.width, size.width))
        }

        fn draw(
            &self,
            _tree: &widget::Tree,
            renderer: &mut Renderer,
            _theme: &Theme,
            _style: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            _viewport: &Rectangle,
        ) {
            use iced::advanced::graphics::mesh::{self, Mesh, SolidVertex2D};
            use iced::advanced::Renderer as _;

            let bounds = layout.bounds();

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
                if let Some(cursor_position) = cursor.position_in(bounds) {
                    [cursor_position.x, cursor_position.y]
                } else {
                    [bounds.width / 2.0, bounds.height / 2.0]
                }
            };

            let posn_tl = [0.0, 0.0];
            let posn_t = [bounds.width / 2.0, 0.0];
            let posn_tr = [bounds.width, 0.0];
            let posn_r = [bounds.width, bounds.height / 2.0];
            let posn_br = [bounds.width, bounds.height];
            let posn_b = [(bounds.width / 2.0), bounds.height];
            let posn_bl = [0.0, bounds.height];
            let posn_l = [0.0, bounds.height / 2.0];

            let mesh = Mesh::Solid {
                size: bounds.size(),
                buffers: mesh::Indexed {
                    vertices: vec![
                        SolidVertex2D {
                            position: posn_center,
                            color: color::pack([1.0, 1.0, 1.0, 1.0]),
                        },
                        SolidVertex2D {
                            position: posn_tl,
                            color: color::pack(color_r),
                        },
                        SolidVertex2D {
                            position: posn_t,
                            color: color::pack(color_o),
                        },
                        SolidVertex2D {
                            position: posn_tr,
                            color: color::pack(color_y),
                        },
                        SolidVertex2D {
                            position: posn_r,
                            color: color::pack(color_g),
                        },
                        SolidVertex2D {
                            position: posn_br,
                            color: color::pack(color_gb),
                        },
                        SolidVertex2D {
                            position: posn_b,
                            color: color::pack(color_b),
                        },
                        SolidVertex2D {
                            position: posn_bl,
                            color: color::pack(color_i),
                        },
                        SolidVertex2D {
                            position: posn_l,
                            color: color::pack(color_v),
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

            renderer.with_translation(
                Vector::new(bounds.x, bounds.y),
                |renderer| {
                    renderer.draw_mesh(mesh);
                },
            );
        }
    }

    impl<'a, Message> From<Rainbow> for Element<'a, Message, Renderer> {
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
