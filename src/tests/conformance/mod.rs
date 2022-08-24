use std::net::{IpAddr, Ipv4Addr};

use crate::{
    protocol::codecs::binary::BinaryMessage,
    setup::{config::ZIGGURAT_CONFIG, node::NodeBuilder},
    tools::{config::TestConfig, synth_node::SyntheticNode},
};

mod handshake;
mod query;

async fn perform_response_test(
    config: TestConfig,
    response_check: &dyn Fn(&BinaryMessage) -> bool,
) {
    // Build and start Ripple node
    let mut node = NodeBuilder::new(
        home::home_dir()
            .expect("Can't find home directory")
            .join(ZIGGURAT_CONFIG),
        IpAddr::V4(Ipv4Addr::LOCALHOST),
    )
    .unwrap()
    .build()
    .await
    .unwrap();

    // Start synth node and connect to Ripple
    let mut synth_node = SyntheticNode::new(&config).await;
    synth_node.connect(node.addr()).await.unwrap();

    // Send the query message (if present)
    config
        .synth_node_config
        .initial_message
        .map(|message| synth_node.unicast(node.addr(), message).unwrap());

    // Wait for a response and perform the given check for it
    assert!(synth_node.expect_message(response_check).await);

    // Shutdown both nodes
    synth_node.shut_down().await;
    node.stop().unwrap();
}
