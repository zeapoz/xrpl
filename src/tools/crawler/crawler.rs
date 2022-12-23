use std::{net::SocketAddr, str::FromStr, sync::Arc};

use pea2pea::protocols::{Handshake, Reading, Writing};
use tokio::{
    sync::mpsc::{channel, Receiver},
    time::{timeout, Instant},
};
use tracing::{error, info, trace, warn};
use ziggurat_xrpl::{
    protocol::{
        codecs::message::{BinaryMessage, Payload},
        proto::TmEndpoints,
    },
    setup::constants::CONNECTION_TIMEOUT,
    tools::inner_node::InnerNode,
};

use crate::network::KnownNetwork;

pub(super) struct Crawler {
    node: InnerNode,
    known_network: Arc<KnownNetwork>,
    receiver: Receiver<(SocketAddr, BinaryMessage)>,
}

impl Crawler {
    pub(super) async fn new() -> Self {
        let (sender, receiver) = channel(1024);
        let node = InnerNode::new(&Default::default(), sender).await;
        node.enable_handshake().await;
        node.enable_reading().await;
        node.enable_writing().await;
        let known_network = Arc::new(Default::default());
        Self {
            node,
            known_network,
            receiver,
        }
    }

    pub(super) async fn start_processing(&mut self) {
        while let Some((addr, message)) = self.receiver.recv().await {
            if let Payload::TmEndpoints(endpoints) = message.payload {
                self.process_endpoints_message(addr, endpoints).await;
            }
        }
    }

    async fn process_endpoints_message(&mut self, addr: SocketAddr, endpoints: TmEndpoints) {
        info!(
            "Received endpoints from {}: {:?}",
            addr, endpoints.endpoints_v2
        );
        for endpoint in endpoints.endpoints_v2 {
            match SocketAddr::from_str(&endpoint.endpoint) {
                Ok(peer) => {
                    self.process_peer(addr, endpoint.hops, peer).await;
                }
                Err(e) => {
                    error!("invalid address from {}: {}", addr, e);
                }
            }
        }
    }

    async fn process_peer(&mut self, addr: SocketAddr, hops: u32, peer: SocketAddr) {
        self.known_network.insert_node(peer).await;
        if hops == 0 {
            self.known_network.insert_connection(addr, peer).await;
        }
        self.get_peers(peer).await;
    }

    pub(super) async fn get_peers(&self, addr: SocketAddr) {
        let node = self.node.clone();
        let network = self.known_network.clone();

        tokio::spawn(async move {
            network.insert_node(addr).await;
            let start = Instant::now();
            if connect_node(node, addr).await {
                network.set_connected(addr, start.elapsed()).await;
            }
        });
    }
}

async fn connect_node(node: InnerNode, addr: SocketAddr) -> bool {
    let connection_result = timeout(CONNECTION_TIMEOUT, node.connect(addr)).await;
    match connection_result {
        Ok(response) => match response {
            Ok(_) => {
                trace!("connected to {}", addr);
                true
            }
            Err(e) => {
                warn!("unable to connect to {} due to {}", addr, e);
                false
            }
        },
        Err(_) => {
            warn!("unable to connect to {} due to timeout", addr);
            false
        }
    }
}
