//! Contains test with ping queries.
//! Queries and expected replies:
//!
//!     - mtPING (with PingType::PtPing) -> mtPING (with PingType::PtPong)

use std::time::Duration;

use rand::{thread_rng, RngCore};
use tempfile::TempDir;
use tokio::time::{sleep, Instant};
use ziggurat_core_utils::err_constants::{
    ERR_NODE_BUILD, ERR_SYNTH_CONNECT, ERR_SYNTH_UNICAST, ERR_TEMPDIR_NEW,
};

use crate::{
    protocol::{
        codecs::message::{BinaryMessage, Payload},
        proto::{tm_ping::PingType, TmPing},
    },
    setup::node::{Node, NodeType},
    tests::conformance::{perform_expected_message_test, TestConfig},
    tools::synth_node::SyntheticNode,
};

const EXPECTED_PING_MESSAGE_TIMEOUT: Duration = Duration::from_secs(62);

#[tokio::test]
#[allow(non_snake_case)]
async fn c003_t1_TM_PING_expect_pong() {
    // ZG-CONFORMANCE-003
    // Send `ping` message
    let seq = thread_rng().next_u32();

    let payload = Payload::TmPing(TmPing {
        r#type: PingType::PtPing as i32,
        seq: Some(seq),
        ping_time: None,
        net_time: None,
    });
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
    // Wait for reply
    perform_expected_message_test(TestConfig::default().with_initial_message(payload), &check)
        .await;
}

#[tokio::test]
#[allow(non_snake_case)]
async fn c003_t2_TM_PING_expect_ping() {
    // ZG-CONFORMANCE-003

    // Create a rippled node.
    let target = TempDir::new().expect(ERR_TEMPDIR_NEW);
    let mut node = Node::builder()
        .start(target.path(), NodeType::Stateful)
        .await
        .expect(ERR_NODE_BUILD);

    // Create a synthetic node and connect it to the node.
    let mut synth_node = SyntheticNode::new(&Default::default()).await;
    synth_node
        .connect(node.addr())
        .await
        .expect(ERR_SYNTH_CONNECT);

    // Wait for ping message so that we can respond with correct `pong`.
    let start = Instant::now();
    let seq = loop {
        if let Ok((_, message)) = synth_node
            .recv_message_timeout(Duration::from_secs(1))
            .await
        {
            match message.payload {
                Payload::TmPing(TmPing {
                    r#type: r_type,
                    seq: Some(seq),
                    ..
                }) if r_type == PingType::PtPing as i32 => break seq,
                _ => {}
            }
        }
        if start.elapsed() > EXPECTED_PING_MESSAGE_TIMEOUT {
            panic!("no ping request within specified timeout");
        }
    };

    // Send `pong` response.
    let response = Payload::TmPing(TmPing {
        r#type: PingType::PtPong as i32,
        seq: Some(seq),
        ping_time: None,
        net_time: None,
    });
    synth_node
        .unicast(node.addr(), response)
        .expect(ERR_SYNTH_UNICAST);

    // Assert that we're still connected after given timeout.
    sleep(EXPECTED_PING_MESSAGE_TIMEOUT).await;
    assert!(synth_node.is_connected(node.addr()));

    // Shutdown both nodes
    synth_node.shut_down().await;
    node.stop().unwrap();
}

#[tokio::test]
#[allow(non_snake_case)]
async fn c003_t3_TM_PING_send_pong() {
    // ZG-CONFORMANCE-003

    // Create a rippled node.
    let target = TempDir::new().expect(ERR_TEMPDIR_NEW);
    let mut node = Node::builder()
        .start(target.path(), NodeType::Stateful)
        .await
        .expect(ERR_NODE_BUILD);

    // Create a synthetic node and connect it to rippled.
    let synth_node = SyntheticNode::new(&Default::default()).await;
    synth_node
        .connect(node.addr())
        .await
        .expect(ERR_SYNTH_CONNECT);

    // Send unsolicited `pong` response.
    synth_node
        .unicast(
            node.addr(),
            Payload::TmPing(TmPing {
                r#type: PingType::PtPong as i32,
                seq: Some(42),
                ping_time: None,
                net_time: None,
            }),
        )
        .expect(ERR_SYNTH_UNICAST);
    sleep(2 * EXPECTED_PING_MESSAGE_TIMEOUT).await;
    assert!(!synth_node.is_connected(node.addr()));

    // Shutdown both nodes
    synth_node.shut_down().await;
    node.stop().unwrap();
}
