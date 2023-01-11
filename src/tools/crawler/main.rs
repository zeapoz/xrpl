use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use clap::Parser;
use futures_util::future::pending;
use reqwest::Client;
use tracing::info;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

use crate::{
    args::Args,
    crawler::Crawler,
    metrics::NetworkSummary,
    rpc::{initialize_rpc_server, RpcContext},
};

mod args;
mod crawl;
mod crawler;
mod metrics;
mod network;
mod rpc;

const CRAWLER_TIMEOUT: Duration = Duration::from_secs(10);

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

    let summary_snapshot = Arc::new(Mutex::new(NetworkSummary::default()));
    let _rpc_handle = if let Some(addr) = args.rpc_addr {
        let rpc_context = RpcContext::new(summary_snapshot.clone());
        let rpc_handle = initialize_rpc_server(addr, rpc_context).await;
        Some(rpc_handle)
    } else {
        None
    };

    info!("Crawler starting with args: {:?}", args);
    let crawler = Crawler::new().await;

    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(CRAWLER_TIMEOUT)
        .build()
        .expect("unable to build the web client");

    for addr in args.seed_addrs {
        crawler::crawl(client.clone(), addr, crawler.known_network.clone()).await;
    }
    pending::<()>().await;
}
