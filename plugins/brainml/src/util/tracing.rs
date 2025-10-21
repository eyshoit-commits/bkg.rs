use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::EnvFilter;

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_timer(UtcTime::rfc_3339())
        .json()
        .init();
}
