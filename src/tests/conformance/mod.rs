use crate::{
    protocol::codecs::binary::{BinaryMessage, Payload},
    setup::node::Node,
    tools::synth_node::SyntheticNode,
};

mod handshake;
mod query;

async fn perform_query_response_test(
    query_msg: Payload,
    response_check: &dyn Fn(&BinaryMessage) -> bool,
) {
    // Start Ripple node
    let mut node = Node::start_with_peers(vec![]).await.unwrap();

    // Start synth node and connect to Ripple
    let mut synth_node = SyntheticNode::start().await.unwrap();
    synth_node.connect(node.addr()).await.unwrap();

    // Send the query message
    synth_node.unicast(node.addr(), query_msg).unwrap();

    // Wait for a response and perform the given check for it
    assert!(synth_node.expect_message(response_check).await);

    // Shutdown both nodes
    synth_node.shut_down().await;
    node.stop().unwrap();
}
