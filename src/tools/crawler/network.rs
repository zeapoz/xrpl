use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    net::SocketAddr,
    time::Duration,
};

use tokio::{sync::RwLock, time::Instant};
use tracing::{error, info};

#[derive(Default)]
pub struct KnownNetwork {
    nodes: RwLock<HashMap<SocketAddr, KnownNode>>,
    connections: RwLock<HashSet<KnownConnection>>,
}

impl KnownNetwork {
    pub(super) async fn insert_node(&self, addr: SocketAddr) {
        let mut nodes = self.nodes.write().await;
        if !nodes.contains_key(&addr) {
            nodes.insert(addr, KnownNode::default());
            info!("Known nodes: {}", nodes.len());
        }
    }

    pub(super) async fn insert_connection(&self, addr: SocketAddr, peer: SocketAddr) {
        let mut connections = self.connections.write().await;
        connections.insert(KnownConnection::new(addr, peer));
    }

    pub(super) async fn set_connected(&self, addr: SocketAddr, handshake_time: Duration) {
        let mut nodes = self.nodes.write().await;
        if !nodes.contains_key(&addr) {
            error!("set_connected(): addr not found ({})", addr);
        } else {
            nodes.get_mut(&addr).unwrap().handshake_time = Some(handshake_time);
        }
    }
}

/// A connection found in the network.
#[derive(Debug, Eq, Copy, Clone)]
pub struct KnownConnection {
    /// One of the two sides of a connection.
    pub a: SocketAddr,
    /// The other side of a connection.
    pub b: SocketAddr,
    /// The timestamp of the last time the connection was seen.
    pub last_seen: Instant,
}

impl Hash for KnownConnection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let (a, b) = (self.a, self.b);

        // This ensures the hash is the same for (a, b) as it is for (b, a).
        match a.cmp(&b) {
            Ordering::Greater => {
                b.hash(state);
                a.hash(state);
            }
            _ => {
                a.hash(state);
                b.hash(state);
            }
        }
    }
}

impl KnownConnection {
    pub fn new(a: SocketAddr, b: SocketAddr) -> Self {
        Self {
            a,
            b,
            last_seen: Instant::now(),
        }
    }
}

impl PartialEq for KnownConnection {
    fn eq(&self, other: &Self) -> bool {
        let (a, b) = (self.a, self.b);
        let (c, d) = (other.a, other.b);

        a == d && b == c || a == c && b == d
    }
}

/// A node encountered in the network or obtained from one of the peers.
#[derive(Debug, Default, Clone)]
pub struct KnownNode {
    // // The address is omitted, as it's a key in the owning HashMap.
    // /// The last time the node was successfully connected to.
    // pub last_connected: Option<Instant>,
    // /// The time it took to complete a connection.
    pub handshake_time: Option<Duration>,
    // /// The node's user agent.
    // pub user_agent: Option<VarStr>,
    // /// The number of subsequent connection errors.
    // pub connection_failures: u8,
}
