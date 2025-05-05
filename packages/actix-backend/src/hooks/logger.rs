use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Initialize the global tracing subscriber with filters
pub fn setup_logger() {
    let filter = Targets::new()
        .with_target("html5ever", LevelFilter::OFF)
        .with_target("markup5ever", LevelFilter::OFF)
        .with_default(LevelFilter::INFO);

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();
}
