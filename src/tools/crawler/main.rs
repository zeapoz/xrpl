use std::{
    num::NonZeroU32,
    sync::{Arc, Mutex},
    time::Duration,
};

use clap::Parser;
use futures_util::future::pending;
use governor::{
    clock::{QuantaClock, QuantaInstant},
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    Jitter, Quota, RateLimiter,
};
use reqwest::Client;
use tracing::info;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use ziggurat_core_crawler::summary::NetworkSummary;

use crate::{
    args::Args,
    crawler::Crawler,
    network::update_summary_snapshot_task,
    rpc::{initialize_rpc_server, RpcContext},
};

mod args;
mod crawl;
mod crawler;
mod metrics;
mod network;
mod rpc;

const CRAWLER_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_REQUESTS_PER_SEC: u32 = 25;
const JITTER_MAX_SEC: u64 = 30;

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

struct Limiter {
    limiter: RateLimiter<NotKeyed, InMemoryState, QuantaClock, NoOpMiddleware<QuantaInstant>>,
    jitter: Jitter,
}

impl Limiter {
    /// Wrapper function around `governor::RateLimiter::until_ready_with_jitter`, using
    /// the self contained `Jitter`.
    async fn until_ready(&self) {
        self.limiter.until_ready_with_jitter(self.jitter).await;
    }
}

impl Default for Limiter {
    fn default() -> Self {
        Self {
            limiter: RateLimiter::direct(Quota::per_second(
                NonZeroU32::new(MAX_REQUESTS_PER_SEC).unwrap(),
            )),
            jitter: Jitter::up_to(Duration::from_secs(JITTER_MAX_SEC)),
        }
    }
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
    let limiter = Arc::new(Limiter::default());

    tokio::spawn(update_summary_snapshot_task(
        crawler.known_network.clone(),
        summary_snapshot,
    ));
    for addr in args.seed_addrs {
        crawler::crawl(
            client.clone(),
            limiter.clone(),
            addr.ip(),
            Some(addr.port()),
            crawler.known_network.clone(),
        )
        .await;
    }
    pending::<()>().await;
}
