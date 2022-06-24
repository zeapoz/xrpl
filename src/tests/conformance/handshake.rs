use pea2pea::{protocols::Handshake, Pea2Pea};

use crate::{setup::node::Node, tools::synthetic_node::SyntheticNode};

#[tokio::test]
async fn handshake_when_node_receives_connection() {
    // crate::tools::synthetic_node::enable_tracing();

    let mut node = Node::new().unwrap();
    node.log_to_stdout(false).start().unwrap();

    // Start synthetic node.
    let node_config = pea2pea::Config {
        listener_ip: Some("127.0.0.1".parse().unwrap()),
        desired_listening_port: Some(12345),
        allow_random_port: false,
        ..Default::default()
    };

    // TODO: replace with a connection from the node to signal readiness.
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let synth_node = SyntheticNode::new(node_config).await;
    synth_node.enable_handshake().await;
    synth_node.node().connect(node.addr()).await.unwrap();

    // This is only set post-handshake.
    assert!(synth_node.node().is_connected(node.addr()));

    // Gracefully shut down the nodes.
    node.stop().unwrap();
}
