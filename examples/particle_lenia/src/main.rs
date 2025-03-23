use iced_lenia::{HEIGHT, WIDTH, subscription, update, view};

fn main() -> iced::Result {
    iced::application("Particle Lenia Simulation", update, view)
        .subscription(subscription)
        .window_size((WIDTH, HEIGHT))
        .run_with(|| {
            // Initialize the application state and return it with an empty task
            let state = iced_lenia::ParticleLenia::new();
            (state, iced::Task::none())
        })
}
