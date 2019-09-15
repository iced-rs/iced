#[macro_use]
extern crate criterion;

use criterion::Criterion;
use criterion::black_box;

use iced::{UserInterface, Cache, Column};
use iced_wgpu::Renderer;

fn criterion_benchmark(c: &mut Criterion) {
    let counter = Counter::new();
    let cache = Cache::new();
    let renderer = Renderer::new();

    c.bench_function("UserInterface::build", |b| b.iter(|| black_box(UserInterface::build(
        counter.view(),
        cache.clone(),
        &renderer,
    ))));
}

mod iced_wgpu {
    pub struct Renderer;
    impl Renderer {
        pub fn new() -> Self { Renderer }
    }
}

pub struct Counter;
impl Counter {
    pub fn new() -> Self { Counter }
    pub fn view(&self) -> Column<(), Renderer> {
        Column::new()
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
