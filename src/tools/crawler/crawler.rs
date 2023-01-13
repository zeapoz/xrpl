use std::{net::SocketAddr, str::FromStr, sync::Arc, time::Duration};

use futures_util::{future::BoxFuture, FutureExt};
use pea2pea::protocols::Handshake;
use reqwest::Client;
use tokio::time::sleep;
use tracing::{trace, warn};
use ziggurat_xrpl::tools::inner_node::InnerNode;

use crate::{
    crawl::{get_crawl_response, CrawlResponse, Peer},
    network::KnownNetwork,
};

const DELAY_AFTER_CONNECTION: Duration = Duration::from_secs(60 * 5);

pub(super) struct Crawler {
    pub(super) known_network: Arc<KnownNetwork>,
}

impl Crawler {
    pub(super) async fn new() -> Self {
        Self {
            known_network: Default::default(),
        }
    }
}

/// Spawns a tokio's task to crawl given address. After receiving the response it will
/// process it and start more crawl tasks recursively.
pub(super) fn crawl(
    client: Client,
    addr: SocketAddr,
    known_network: Arc<KnownNetwork>,
) -> BoxFuture<'static, ()> {
    // Wrapped in box to allow for async recursion.
    async move {
        tokio::spawn(async move {
            trace!("Crawling {}", addr);
            if known_network.new_node(addr).await {
                loop {
                    tokio::spawn(try_handshake(addr, known_network.clone()));
                    let success = try_crawling(client.clone(), addr, known_network.clone()).await;
                    if !success {
                        let failures = known_network.increase_connection_failures(addr).await;
                        if failures == u8::MAX {
                            warn!("Giving up connecting to {}", addr);
                            break;
                        }
                    }
                    // Even if connection was successful try again after a while to update peers.
                    sleep(DELAY_AFTER_CONNECTION).await;
                }
            }
        });
    }
    .boxed()
}

async fn try_handshake(addr: SocketAddr, known_network: Arc<KnownNetwork>) {
    let (sender, _receiver) = tokio::sync::mpsc::channel(1024);
    let node = InnerNode::new(&Default::default(), sender).await;
    node.enable_handshake().await;
    if node.connect(addr).await.is_ok() {
        known_network.set_handshake_successful(addr, true).await;
        trace!("Successful handshake to {}", addr);
    } else {
        known_network.set_handshake_successful(addr, false).await;
        warn!("Unsuccessful handshake to {}", addr);
    }
    node.shut_down().await;
}

async fn try_crawling(client: Client, addr: SocketAddr, known_network: Arc<KnownNetwork>) -> bool {
    match get_crawl_response(client.clone(), addr).await {
        Ok((response, connecting_time)) => {
            let addresses = extract_known_nodes(&response).await;
            known_network
                .update_stats(addr, connecting_time, response.server.build_version)
                .await;
            known_network.insert_connections(addr, &addresses).await;
            for addr in addresses {
                crawl(client.clone(), addr, known_network.clone()).await;
            }
            true
        }
        Err(e) => {
            warn!("Unable to get crawl response from {}: {:?}", addr, e);
            false
        }
    }
}

/// Extract addresses from /crawl response.
async fn extract_known_nodes(response: &CrawlResponse) -> Vec<SocketAddr> {
    response
        .overlay
        .active
        .iter()
        .filter_map(|peer| parse_peer_addr(peer))
        .collect()
}

/// Tries to parse address information from response.
/// On success returns Some(SocketAddr) on failure returns None.
fn parse_peer_addr(peer: &Peer) -> Option<SocketAddr> {
    SocketAddr::from_str(format!("{}:{}", peer.ip.as_ref()?, peer.port().ok()?).as_str()).ok()
}
