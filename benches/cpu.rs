#![allow(missing_docs)]
use criterion::{Bencher, Criterion, criterion_group, criterion_main};

use iced::border;
use iced::mouse;
use iced::widget::{canvas, center_y, column, container, row, scrollable, space, text};
use iced::{Center, Color, Element, Fill, Length, Never, Point, Rectangle, Size, Theme};
use iced_renderer::Renderer;
use iced_renderer::core::renderer::{self, Headless as _};
use iced_runtime::UserInterface;
use iced_runtime::user_interface;

use iced_futures::futures::executor;

criterion_main!(benches);
criterion_group!(benches, benchmark);

const VIEWPORT: Size<f32> = Size::new(512.0, 512.0);
const SCALE: f32 = 2.0;

pub fn benchmark(c: &mut Criterion) {
    let mut renderer = executor::block_on(Renderer::new(
        renderer::Settings::default(),
        Some("software"),
    ))
    .expect("software renderer must be available");

    let _ = c
        .bench_function("cpu — ipsum", |b| {
            draw(b, &mut renderer, ipsum());
        })
        .bench_function("cpu — application", |b| {
            draw(b, &mut renderer, application());
        });
}

fn draw(bencher: &mut Bencher<'_>, renderer: &mut Renderer, view: Element<'static, Never>) {
    let mut ui = UserInterface::build(view, VIEWPORT, user_interface::Cache::new(), renderer);

    bencher.iter(|| {
        ui.draw(
            renderer,
            &Theme::Dark,
            &renderer::Style::default(),
            mouse::Cursor::Unavailable,
        );

        let _ = renderer.screenshot(
            Size::new(
                (VIEWPORT.width * SCALE) as u32,
                (VIEWPORT.height * SCALE) as u32,
            ),
            SCALE,
            Color::WHITE,
        );
    });
}

fn ipsum() -> Element<'static, Never> {
    text(include_str!("ipsum.txt"))
        .ellipsis(text::Ellipsis::End)
        .into()
}

fn application() -> Element<'static, Never> {
    fn square<'a>(size: impl Into<Length> + Copy) -> Element<'a, Never> {
        struct Square;

        impl canvas::Program<Never> for Square {
            type State = ();

            fn draw(
                &self,
                _state: &Self::State,
                renderer: &Renderer,
                theme: &Theme,
                bounds: Rectangle,
                _cursor: mouse::Cursor,
            ) -> Vec<canvas::Geometry> {
                let mut frame = canvas::Frame::new(renderer, bounds.size());

                let palette = theme.palette();

                frame.fill_rectangle(
                    Point::ORIGIN,
                    bounds.size(),
                    palette.background.strong.color,
                );

                vec![frame.into_geometry()]
            }
        }

        canvas(Square).width(size).height(size).into()
    }

    let header = container(
        row![
            square(40),
            space::horizontal(),
            "Header!",
            space::horizontal(),
            square(40),
        ]
        .padding(10)
        .align_y(Center),
    )
    .style(|theme| {
        let palette = theme.palette();

        container::Style::default().border(border::color(palette.background.strong.color).width(1))
    });

    let sidebar = center_y(
        column!["Sidebar!", square(50), square(50)]
            .spacing(40)
            .padding(10)
            .width(200)
            .align_x(Center),
    )
    .style(container::rounded_box);

    let content = container(
        scrollable(
            column![
                "Content!",
                row((1..10).map(|i| square(if i % 2 == 0 { 80 } else { 160 })))
                    .spacing(20)
                    .align_y(Center)
                    .wrap(),
                "The end"
            ]
            .spacing(40)
            .align_x(Center)
            .width(Fill),
        )
        .height(Fill),
    )
    .padding(10);

    column![header, row![sidebar, content]].into()
}
