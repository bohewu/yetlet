//! Explicit host-owned output. Only the safe reference namespace is enabled.
use tracing_subscriber::{
    filter::{LevelFilter, Targets},
    prelude::*,
};
pub(super) fn initialize() -> Result<(), ()> {
    tracing_subscriber::registry()
        .with(Targets::new().with_target(
            yetlet_lab_operations_board::REQUEST_TRACING_TARGET,
            LevelFilter::INFO,
        ))
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_current_span(true)
                .with_span_list(true)
                .with_file(false)
                .with_line_number(false)
                .with_writer(std::io::stderr),
        )
        .try_init()
        .map_err(|_| ())
}
