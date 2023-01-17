use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use serde::Serialize;
use spectre::{edge::Edge, graph::Graph};

use crate::network::{KnownNetwork, KnownNode};

/// The elapsed time before a connection should be regarded as inactive.
pub const LAST_SEEN_CUTOFF: u64 = 10 * 60;

#[derive(Default)]
pub struct NetworkMetrics {
    graph: Graph<SocketAddr>,
}

impl NetworkMetrics {
    /// Updates the network graph with new connections.
    pub(super) async fn update_graph(&mut self, known_network: Arc<KnownNetwork>) {
        for connection in known_network.connections().await {
            let edge = Edge::new(connection.a, connection.b);
            if connection.last_seen.elapsed().as_secs() > LAST_SEEN_CUTOFF {
                self.graph.remove(&edge);
            } else {
                self.graph.insert(edge);
            }
        }
    }
}

#[derive(Default, Clone, Serialize)]
pub(super) struct NetworkSummary {
    num_known_nodes: usize,
    num_good_nodes: usize,
    num_known_connections: usize,
    density: f64,
    degree_centrality_delta: f64,
    avg_degree_centrality: u64,
    node_ids: Vec<String>,
}

impl NetworkSummary {
    /// Builds a new [NetworkSummary] out of current state of [KnownNetwork]
    pub(super) async fn new(
        known_network: Arc<KnownNetwork>,
        metrics: &mut NetworkMetrics,
    ) -> Self {
        let nodes = known_network.nodes().await;
        let connections = known_network.connections().await;
        let good_nodes = get_good_nodes(&nodes);

        let node_ids = get_node_ids(&good_nodes);
        // Procure metrics from the graph.
        let density = metrics.graph.density();
        let degree_centrality_delta = metrics.graph.degree_centrality_delta();
        let degree_centralities = metrics.graph.degree_centrality();
        let avg_degree_centrality = degree_centralities.values().map(|v| *v as u64).sum::<u64>()
            / degree_centralities.len() as u64;

        Self {
            num_known_nodes: nodes.len(),
            num_good_nodes: good_nodes.len(),
            num_known_connections: connections.len(),
            density,
            degree_centrality_delta,
            avg_degree_centrality,
            node_ids,
        }
    }
}

fn get_node_ids(good_nodes: &HashMap<SocketAddr, KnownNode>) -> Vec<String> {
    let good_addresses: Vec<SocketAddr> = good_nodes.keys().cloned().collect();
    let node_ids = good_addresses
        .iter()
        .map(|addr| md5::compute(addr.to_string()))
        .map(|digest| format!("{:x}", digest))
        .map(|hex| hex[0..12].to_string())
        .collect::<Vec<_>>();
    node_ids
}

fn get_good_nodes(nodes: &HashMap<SocketAddr, KnownNode>) -> HashMap<SocketAddr, KnownNode> {
    let good_nodes: HashMap<_, _> = nodes
        .clone()
        .into_iter()
        .filter(|(_, node)| node.last_connected.is_some())
        .collect();
    good_nodes
}
