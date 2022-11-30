use std::time::Duration;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

#[cfg(feature = "tracing-chrome")]
use {
    tracing_chrome::FlushGuard,
    tracing_subscriber::fmt::{format::DefaultFields, FormattedFields},
};

pub use tracing::{info_span, instrument};

/// Profiler state. This will likely need to be updated or reworked when adding new tracing backends.
pub struct Profiler {
    #[cfg(feature = "tracing-chrome")]
    /// [`FlushGuard`] must not be dropped until the application scope is dropped for accurate tracing.
    _guard: FlushGuard,
}

pub fn init() -> Profiler {
    // Registry stores the spans & generates unique span IDs
    let subscriber = Registry::default();

    #[cfg(feature = "tracing-chrome")]
    let (chrome_layer, guard) = {
        let mut layer = tracing_chrome::ChromeLayerBuilder::new();

        // Optional configurable env var: CHROME_TRACE_FILE=/path/to/trace_file/file.json,
        // for uploading to chrome://tracing (old) or ui.perfetto.dev (new).
        if let Ok(path) = std::env::var("CHROME_TRACE_FILE") {
            layer = layer.file(path);
        } else if let Ok(current_dir) = std::env::current_dir() {
            let time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(Duration::from_millis(0))
                .as_millis();

            let trace_file_name = current_dir
                .file_name()
                .map(|file_dir| {
                    format!(
                        "{}_trace_{}.json",
                        file_dir.to_str().unwrap_or("trace"),
                        time
                    )
                })
                .unwrap_or_else(|| "trace.json".to_string());

            let path = format!(
                "{}/{}",
                current_dir.to_str().expect("Invalid path"),
                trace_file_name
            );

            layer = layer.file(path);
        } else {
            layer = layer.file(env!("CARGO_MANIFEST_DIR"))
        }

        let (chrome_layer, guard) = layer
            .name_fn(Box::new(|event_or_span| match event_or_span {
                tracing_chrome::EventOrSpan::Event(event) => {
                    event.metadata().name().into()
                }
                tracing_chrome::EventOrSpan::Span(span) => {
                    if let Some(fields) = span
                        .extensions()
                        .get::<FormattedFields<DefaultFields>>()
                    {
                        format!(
                            "{}: {}",
                            span.metadata().name(),
                            fields.fields.as_str()
                        )
                    } else {
                        span.metadata().name().into()
                    }
                }
            }))
            .build();

        (chrome_layer, guard)
    };

    let fmt_layer = tracing_subscriber::fmt::Layer::default();
    let subscriber = subscriber.with(fmt_layer);

    #[cfg(feature = "tracing-chrome")]
    let subscriber = subscriber.with(chrome_layer);

    // create dispatcher which will forward span events to the subscriber
    // this can only be set once or will panic
    tracing::subscriber::set_global_default(subscriber)
        .expect("Profiler could not set the global default subscriber.");

    Profiler {
        #[cfg(feature = "tracing-chrome")]
        _guard: guard,
    }
}
