use clap::Parser;
use tracing::debug;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

use crate::args::Args;

mod args;

fn start_logger(default_level: LevelFilter) {
    let filter = match EnvFilter::try_from_default_env() {
        Ok(filter) => filter
            .add_directive("tokio_util=off".parse().unwrap())
            .add_directive("mio=off".parse().unwrap()),
        _ => EnvFilter::default()
            .add_directive(default_level.into())
            .add_directive("tokio_util=off".parse().unwrap())
            .add_directive("mio=off".parse().unwrap()),
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

#[tokio::main]
async fn main() {
    start_logger(LevelFilter::DEBUG);
    let args = Args::parse();
    debug!("Crawler starting with args: {:?}", args);
}
