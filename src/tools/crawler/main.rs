use clap::Parser;
use tracing::info;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

use crate::{args::Args, crawler::Crawler};

mod args;
mod crawler;
mod network;

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
    start_logger(LevelFilter::INFO);
    let args = Args::parse();

    info!("Crawler starting with args: {:?}", args);
    let mut crawler = Crawler::new().await;

    for addr in args.seed_addrs {
        crawler.get_peers(addr).await;
    }

    crawler.start_processing().await;
}
