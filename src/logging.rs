use ansi_term::enable_ansi_support;
pub use tracing::{debug, error, info, instrument, warn};

use tracing_subscriber::FmtSubscriber;

/// Initializes the subscriber.
/// Run only once at startup
pub(super) fn init_log() {
    #[cfg(debug_assertions)]
    let level = "debug";

    #[cfg(not(debug_assertions))]
    let level = "info";

    let crate_name = env!("CARGO_PKG_NAME");

    let subscriber = FmtSubscriber::builder()
        .compact()
        .with_env_filter(format!("{crate_name}={level},hyper=info",))
        .with_ansi(enable_ansi_support().is_ok())
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();
}
