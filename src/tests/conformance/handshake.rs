use crate::{
    setup::node::{Node, CONNECTION_TIMEOUT},
    tools::synth_node::SyntheticNode,
    wait_until,
};

#[tokio::test]
async fn handshake_when_node_receives_connection() {
    // ZG-CONFORMANCE-001

    // crate::tools::synthetic_node::enable_tracing();

    // Start the Ripple node
    let mut node = Node::start_with_peers(vec![]).await.unwrap();

    // Start synthetic node.
    let synth_node = SyntheticNode::start().await.unwrap();
    synth_node.connect(node.addr()).await.unwrap();

    // This is only set post-handshake.
    assert_eq!(synth_node.num_connected(), 1);
    assert!(synth_node.is_connected(node.addr()));

    // Shutdown both nodes
    synth_node.shut_down().await;
    node.stop().unwrap();
}

#[tokio::test]
async fn handshake_when_node_initiates_connection() {
    // ZG-CONFORMANCE-002

    // crate::tools::synthetic_node::enable_tracing();

    // Start synthetic node.
    let synth_node = SyntheticNode::start().await.unwrap();

    // Start the Ripple node and set the synth node as an initial peer.
    let mut node = Node::start_with_peers(vec![synth_node.listening_addr().unwrap()])
        .await
        .unwrap();

    wait_until!(CONNECTION_TIMEOUT, synth_node.num_connected() == 1);
    assert!(synth_node.is_connected_ip(node.addr().ip()));

    // Shutdown both nodes
    synth_node.shut_down().await;
    node.stop().unwrap();
}
