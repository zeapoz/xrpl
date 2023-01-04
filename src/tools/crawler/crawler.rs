use std::{net::SocketAddr, str::FromStr, sync::Arc};

use futures_util::{future::BoxFuture, FutureExt};
use reqwest::Client;
use tracing::{trace, warn};

use crate::{
    crawl::{get_crawl_response, CrawlResponse, Peer},
    network::KnownNetwork,
};

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
                    }
                    Err(e) => {
                        warn!("Unable to get crawl response from {}: {:?}", addr, e);
                        // TODO if it's connection refused or timeout: retry connection a few time after a while
                    }
                }
            }
        });
    }
    .boxed()
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
