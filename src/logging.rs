use std::path::Path;
use tracing::{Level, Subscriber, subscriber::set_global_default};
use tracing_appender::rolling::{self};
use tracing_subscriber::{EnvFilter, Registry, fmt, layer::SubscriberExt};

pub fn get_subscriber(
    filter: &str,
    log_directory: impl AsRef<Path>,
) -> impl Subscriber + Send + Sync {
    let file_appender = rolling::daily(log_directory, "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| filter.into());

    let file_layer = fmt::Layer::new().with_writer(non_blocking).with_ansi(false);

    let console_layer = fmt::Layer::new().with_writer(std::io::stdout).pretty();

    Registry::default()
        .with(env_filter)
        .with(file_layer)
        .with(console_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    set_global_default(subscriber).expect("Failed to set global subscriber");
}
