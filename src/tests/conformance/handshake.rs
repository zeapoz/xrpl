use std::net::{IpAddr, Ipv4Addr};

use crate::{
    setup::{
        config::ZIGGURAT_DIR,
        node::{NodeBuilder, CONNECTION_TIMEOUT},
    },
    tools::synth_node::SyntheticNode,
    wait_until,
};

#[tokio::test]
async fn handshake_when_node_receives_connection() {
    // ZG-CONFORMANCE-001

    // crate::tools::synthetic_node::enable_tracing();

    // Build and start the Ripple node
    let mut node = NodeBuilder::new(
        home::home_dir()
            .expect("Can't find home directory")
            .join(ZIGGURAT_DIR),
        IpAddr::V4(Ipv4Addr::LOCALHOST),
    )
    .unwrap()
    .build()
    .await
    .unwrap();

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
async fn handshake_when_node_initiates_connection() {
    // ZG-CONFORMANCE-002

    // crate::tools::synthetic_node::enable_tracing();

    // Start synthetic node.
    let synth_node = SyntheticNode::new(&Default::default()).await;

    // Build and start the Ripple node and set the synth node as an initial peer.
    let mut node = NodeBuilder::new(
        home::home_dir()
            .expect("Can't find home directory")
            .join(ZIGGURAT_DIR),
        IpAddr::V4(Ipv4Addr::LOCALHOST),
    )
    .unwrap()
    .initial_peers(vec![synth_node.listening_addr().unwrap()])
    .build()
    .await
    .unwrap();

    wait_until!(CONNECTION_TIMEOUT, synth_node.num_connected() == 1);
    assert!(synth_node.is_connected_ip(node.addr().ip()));

    // Shutdown both nodes
    synth_node.shut_down().await;
    node.stop().unwrap();
}
