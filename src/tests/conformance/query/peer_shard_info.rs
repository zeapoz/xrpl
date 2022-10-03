//! Contains test with peer shard info queries.

use tempfile::TempDir;

use crate::{
    protocol::{
        codecs::binary::{BinaryMessage, Payload},
        proto::{TmGetPeerShardInfoV2, TmPublicKey},
    },
    setup::node::{Node, NodeType},
    tests::conformance::perform_response_test,
    tools::{config::TestConfig, synth_node::SyntheticNode},
};

const KEY_TYPES: &[u8] = &[
    0xED, // ed25519
    0x02, // secp256k1
    0x03, // secp256k1 again as this type key has two correct magic bytes.
];

const INVALID_KEY: u8 = 0x42;
const RELAY_LIMIT: u32 = 3;

#[tokio::test]
#[allow(non_snake_case)]
async fn c005_TM_GET_PEER_SHARD_INFO_V2_node_should_query_for_shard_info_after_handshake() {
    // ZG-CONFORMANCE-005
    let response_check =
        |m: &BinaryMessage| matches!(&m.payload, Payload::TmGetPeerShardInfoV2(..));
    perform_response_test(Default::default(), &response_check).await;
}

#[tokio::test]
#[should_panic]
#[allow(non_snake_case)]
async fn c006_TM_GET_PEER_SHARD_INFO_V2_node_should_not_query_for_shard_info_if_no_handshake() {
    // ZG-CONFORMANCE-006
    let response_check =
        |m: &BinaryMessage| matches!(&m.payload, Payload::TmGetPeerShardInfoV2(..));
    perform_response_test(TestConfig::default().with_handshake(false), &response_check).await;
}

#[tokio::test]
#[allow(non_snake_case)]
async fn c011_TM_GET_PEER_SHARD_INFO_V2_node_should_relay_shard_info() {
    // ZG-CONFORMANCE-011
    for key_type in KEY_TYPES {
        check_relay_for_key_type(*key_type, RELAY_LIMIT - 1).await;
    }
}

#[tokio::test]
#[should_panic]
#[allow(non_snake_case)]
async fn c012_TM_GET_PEER_SHARD_INFO_V2_node_should_not_relay_shard_info_with_invalid_key_type() {
    // ZG-CONFORMANCE-012
    check_relay_for_key_type(INVALID_KEY, RELAY_LIMIT - 1).await;
}

#[tokio::test]
#[should_panic]
#[allow(non_snake_case)]
async fn c013_TM_GET_PEER_SHARD_INFO_V2_node_should_not_relay_shard_info_when_relays_equals_zero() {
    // ZG-CONFORMANCE-013
    check_relay_for_key_type(KEY_TYPES[0], 0).await;
}

#[tokio::test]
#[should_panic]
#[allow(non_snake_case)]
async fn c014_TM_GET_PEER_SHARD_INFO_V2_node_should_not_relay_shard_info_when_relays_above_limit() {
    // ZG-CONFORMANCE-014
    check_relay_for_key_type(KEY_TYPES[0], RELAY_LIMIT + 1).await;
}

async fn check_relay_for_key_type(key_type: u8, relays: u32) {
    // Create node.
    let target = TempDir::new().expect("unable to create TempDir");
    let mut node = Node::builder()
        .log_to_stdout(false)
        .start(target.path(), NodeType::Stateless)
        .await
        .expect("unable to start the rippled node");

    // Create two synthetic nodes and connect them to rippled.
    let synth_node1 = SyntheticNode::new(&Default::default()).await;
    synth_node1
        .connect(node.addr())
        .await
        .expect("unable to connect");
    let mut synth_node2 = SyntheticNode::new(&Default::default()).await;
    synth_node2
        .connect(node.addr())
        .await
        .expect("unable to connect");

    // Create a dummy key with the specified key type.
    let mut key = vec![key_type]; // Place the key type as the first byte.
    key.resize(33, 0x1); // Append 32 bytes serving as a dummy public key.
    let public_key = TmPublicKey { public_key: key };

    // Create payload with given key and relays.
    let payload = Payload::TmGetPeerShardInfoV2(TmGetPeerShardInfoV2 {
        peer_chain: vec![public_key.clone()],
        relays,
    });
    // Send a message from the first synthetic node.
    synth_node1
        .unicast(node.addr(), payload)
        .expect("unable to send message");

    // Ensure that the second synthetic node receives the relayed message.
    // Verify the public key and ensure that the `relays` number got subtracted.
    let check = |m: &BinaryMessage| {
        matches!(&m.payload, Payload::TmGetPeerShardInfoV2(TmGetPeerShardInfoV2{peer_chain, relays: received_relays})
          if peer_chain.get(0) == Some(&public_key) && *received_relays == relays.saturating_sub(1))
    };
    assert!(synth_node2.expect_message(&check).await);

    // Shutdown.
    synth_node1.shut_down().await;
    synth_node2.shut_down().await;
    node.stop().expect("unable to stop rippled node");
}
