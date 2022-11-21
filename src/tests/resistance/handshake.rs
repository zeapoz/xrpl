use tempfile::TempDir;
use tokio::time::sleep;

use crate::{
    setup::{
        constants::CONNECTION_TIMEOUT,
        node::{Node, NodeType},
    },
    tools::{config::TestConfig, synth_node::SyntheticNode},
};

#[allow(non_snake_case)]
#[tokio::test]
async fn r001_t1_HANDSHAKE_reject_if_user_agent_too_long() {
    // ZG-RESISTANCE-001

    // Build and start the Ripple node.
    let target = TempDir::new().expect("Couldn't create a temporary directory");
    let mut node = Node::builder()
        .start(target.path(), NodeType::Stateless)
        .await
        .expect("unable to start the node");

    // Start the first synthetic node with a 'User-Agent' header that's too long.
    let mut test_config = TestConfig::default();
    test_config.synth_node_config.ident = format!("{:8192}", 0);
    let synth_node1 = SyntheticNode::new(&test_config).await;
    // Ensure this connection was rejected by the node.
    assert!(synth_node1.connect(node.addr()).await.is_err());
    assert_eq!(synth_node1.num_connected(), 0);
    assert!(!synth_node1.is_connected(node.addr()));

    // Start the second synthetic node with the default 'User-Agent'.
    let synth_node2 = SyntheticNode::new(&Default::default()).await;
    synth_node2.connect(node.addr()).await.unwrap();
    // Ensure this connection was successful.
    assert_eq!(synth_node2.num_connected(), 1);
    assert!(synth_node2.is_connected(node.addr()));

    // Shutdown all nodes.
    synth_node1.shut_down().await;
    synth_node2.shut_down().await;
    node.stop().unwrap();
}

#[allow(non_snake_case)]
#[tokio::test]
async fn r001_t2_HANDSHAKE_reject_if_server_too_long() {
    // ZG-RESISTANCE-001

    // Start the first synthetic node. Set identification ('Server' header) for the value that's too long.
    let mut test_config = TestConfig::default();
    test_config.synth_node_config.ident = format!("{:8192}", 0);
    let synth_node1 = SyntheticNode::new(&test_config).await;

    // Start the second synthetic node with the default 'Server' header.
    let synth_node2 = SyntheticNode::new(&Default::default()).await;

    // Build and start the Ripple node. Configure its peers such that it connects to the synthetic node above.
    let target = TempDir::new().expect("Couldn't create a temporary directory");
    let mut node = Node::builder()
        .initial_peers(vec![
            synth_node1.listening_addr().unwrap(),
            synth_node2.listening_addr().unwrap(),
        ])
        .start(target.path(), NodeType::Stateless)
        .await
        .expect("unable to start the node");

    // Wait for as long as is usually needed.
    sleep(CONNECTION_TIMEOUT).await;

    // Ensure the connection to the first synthetic node was rejected by the node.
    assert!(!synth_node1.is_connected_ip(node.addr().ip()));

    // Ensure the connection to the second synthetic node was successful.
    assert!(synth_node2.is_connected_ip(node.addr().ip()));

    // Shutdown all nodes.
    synth_node1.shut_down().await;
    synth_node2.shut_down().await;
    node.stop().unwrap();
}
