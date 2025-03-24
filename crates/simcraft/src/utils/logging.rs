use std::sync::Once;
use tracing_subscriber::{fmt, EnvFilter};

static INIT_LOGGING_ONCE: Once = Once::new();

pub fn init_logging_once() {
    INIT_LOGGING_ONCE.call_once(|| {
        match fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .try_init()
        {
            Ok(_) => (),
            Err(_) => eprintln!("Logger already initialized; skipping"),
        }
    });
}
