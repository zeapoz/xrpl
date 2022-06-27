use std::time::Duration;

use pea2pea::{protocols::Handshake, Pea2Pea};

use crate::{setup::node::Node, tools::synthetic_node::SyntheticNode, wait_until};

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(2);

#[tokio::test]
async fn handshake_when_node_receives_connection() {
    // crate::tools::synthetic_node::enable_tracing();

    let mut node = Node::new().unwrap();
    node.log_to_stdout(false).start().unwrap();

    // Start synthetic node.
    let node_config = pea2pea::Config {
        listener_ip: Some("127.0.0.1".parse().unwrap()),
        ..Default::default()
    };

    // TODO: replace with a connection from the node to signal readiness.
    tokio::time::sleep(CONNECTION_TIMEOUT).await;

    let synth_node = SyntheticNode::new(node_config).await;
    synth_node.enable_handshake().await;
    synth_node.node().connect(node.addr()).await.unwrap();

    // This is only set post-handshake.
    assert_eq!(synth_node.node().num_connected(), 1);
    assert!(synth_node.node().is_connected(node.addr()));

    // Gracefully shut down the Ripple node.
    node.stop().unwrap();
}

#[tokio::test]
async fn handshake_when_node_initiates_connection() {
    // crate::tools::synthetic_node::enable_tracing();

    // Start synthetic node.
    let node_config = pea2pea::Config {
        listener_ip: Some("127.0.0.1".parse().unwrap()),
        ..Default::default()
    };

    let synth_node = SyntheticNode::new(node_config).await;
    synth_node.enable_handshake().await;

    // Start the Ripple node and set the synth node as an initial peer.
    let mut node = Node::new().unwrap();
    // TODO: consider implementing a hs! (HashSet::new) macro.
    node.initial_peers(vec![synth_node.node().listening_addr().unwrap()])
        .log_to_stdout(false)
        .start()
        .unwrap();

    wait_until!(CONNECTION_TIMEOUT, synth_node.node().num_connected() == 1);
    assert!(synth_node.is_connected_ip(node.addr().ip()));

    // Gracefully shut down the Ripple node.
    node.stop().unwrap();
}
