//! Contains test with basic queries.
//!
//! Queries and expected replies:
//!
//!     - Ping -> Pong

use rand::{thread_rng, RngCore};

use crate::{
    protocol::{
        codecs::binary::{BinaryMessage, Payload},
        proto::{tm_ping::PingType, TmPing},
    },
    setup::node::Node,
    tools::synth_node::SyntheticNode,
};

#[tokio::test]
async fn should_respond_with_pong_for_ping() {
    // ZG-CONFORMANCE-003

    // Start Ripple node
    let mut node = Node::start_with_peers(vec![]).await.unwrap();

    // Start synth node and connect to Ripple
    let mut synth_node = SyntheticNode::start().await.unwrap();
    synth_node.connect(node.addr()).await.unwrap();

    // Send `ping` message
    let seq = thread_rng().next_u32();

    let payload = Payload::TmPing(TmPing {
        r#type: PingType::PtPing as i32,
        seq: Some(seq),
        ping_time: None,
        net_time: None,
    });

    synth_node.unicast(node.addr(), payload).unwrap();

    // Wait for 'pong' response
    let check = |m: &BinaryMessage| {
        matches!(
            &m.payload,
            // proto file defines 'pong' message as `TmPing` with `r#type` set to [PingType::PtPong]
            Payload::TmPing(TmPing {
                r#type: r_type,
                seq: Some(s),
                ..
            }) if *s == seq && *r_type == PingType::PtPong as i32
        )
    };
    assert!(synth_node.expect_message(&check).await);

    // Shutdown both nodes
    synth_node.shut_down().await;
    node.stop().unwrap();
}
