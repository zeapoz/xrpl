use std::{collections::HashMap, sync::Arc};

use serde::Serialize;

use crate::network::KnownNetwork;

#[derive(Default, Clone, Serialize)]
pub(super) struct NetworkSummary {
    num_known_nodes: usize,
    num_good_nodes: usize,
    num_known_connections: usize,
}

impl NetworkSummary {
    /// Builds a new [NetworkSummary] out of current state of [KnownNetwork]
    pub(super) async fn new(known_network: Arc<KnownNetwork>) -> Self {
        let nodes = known_network.nodes().await;
        let connections = known_network.connections().await;
        let good_nodes: HashMap<_, _> = nodes
            .clone()
            .into_iter()
            .filter(|(_, node)| node.last_connected.is_some())
            .collect();
        Self {
            num_known_nodes: nodes.len(),
            num_good_nodes: good_nodes.len(),
            num_known_connections: connections.len(),
        }
    }
}
