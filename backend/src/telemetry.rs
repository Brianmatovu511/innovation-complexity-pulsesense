use tracing_subscriber::{fmt, EnvFilter};

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,actix_web=info"));

    // Use the default human-readable formatter (no .json()).
    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_level(true)
        .init();
}
