use tempfile::TempDir;

use crate::{
    setup::node::{Node, NodeType},
    tools::{constants::CONNECTION_TIMEOUT, synth_node::SyntheticNode},
    wait_until,
};

#[tokio::test]
async fn c001_handshake_when_node_receives_connection() {
    // ZG-CONFORMANCE-001

    // crate::tools::synth_node::enable_tracing();

    // Build and start the Ripple node
    let target = TempDir::new().expect("Can't build tmp dir");
    let mut node = Node::builder()
        .start(target.path(), NodeType::Stateless)
        .await
        .expect("Unable to start node");

    // Start synthetic node.
    let synth_node = SyntheticNode::new(&Default::default()).await;
    synth_node.connect(node.addr()).await.unwrap();

    // This is only set post-handshake.
    assert_eq!(synth_node.num_connected(), 1);
    assert!(synth_node.is_connected(node.addr()));

    // Shutdown both nodes
    synth_node.shut_down().await;
    node.stop().unwrap();
}

#[tokio::test]
async fn c002_handshake_when_node_initiates_connection() {
    // ZG-CONFORMANCE-002

    // crate::tools::synth_node::enable_tracing();

    // Start synthetic node.
    let synth_node = SyntheticNode::new(&Default::default()).await;

    // Build and start the Ripple node and set the synth node as an initial peer.
    let target = TempDir::new().expect("Can't build tmp dir");
    let mut node = Node::builder()
        .initial_peers(vec![synth_node.listening_addr().unwrap()])
        .start(target.path(), NodeType::Stateless)
        .await
        .expect("Unable to start node");

    wait_until!(CONNECTION_TIMEOUT, synth_node.num_connected() == 1);
    assert!(synth_node.is_connected_ip(node.addr().ip()));

    // Shutdown both nodes
    synth_node.shut_down().await;
    node.stop().unwrap();
}
