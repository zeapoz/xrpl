//! Contains test with peer shard info queries.

use secp256k1::constants::PUBLIC_KEY_SIZE;
use tempfile::TempDir;

use crate::{
    protocol::{
        codecs::message::{BinaryMessage, Payload},
        proto::{TmGetPeerShardInfoV2, TmPublicKey},
    },
    setup::node::{Node, NodeType},
    tests::conformance::PUBLIC_KEY_TYPES,
    tools::{rpc::wait_for_state, synth_node::SyntheticNode},
};

const INVALID_KEY: u8 = 0x42;
const RELAY_LIMIT: u32 = 3;

#[tokio::test]
#[allow(non_snake_case)]
async fn c011_TM_GET_PEER_SHARD_INFO_V2_node_should_relay_shard_info() {
    // ZG-CONFORMANCE-011
    for key_type in PUBLIC_KEY_TYPES {
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
    check_relay_for_key_type(PUBLIC_KEY_TYPES[0], 0).await;
}

#[tokio::test]
#[should_panic]
#[allow(non_snake_case)]
async fn c014_TM_GET_PEER_SHARD_INFO_V2_node_should_not_relay_shard_info_when_relays_above_limit() {
    // ZG-CONFORMANCE-014
    check_relay_for_key_type(PUBLIC_KEY_TYPES[0], RELAY_LIMIT + 1).await;
}

async fn check_relay_for_key_type(key_type: u8, relays: u32) {
    // Create node.
    let target = TempDir::new().expect("unable to create TempDir");
    let mut node = Node::builder()
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
    key.resize(PUBLIC_KEY_SIZE, 0x1); // Append 32 bytes serving as a dummy public key.
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
    node.stop().expect("unable to stop the rippled node");
}

#[tokio::test]
#[allow(non_snake_case)]
async fn c023_TM_PEER_SHARD_INFO_V2_node_should_respond_with_shard_info_if_sharding_enabled() {
    // ZG-CONFORMANCE-023

    // Create a rippled node.
    let target = TempDir::new().expect("unable to create TempDir");
    let mut node = Node::builder()
        .enable_sharding(true)
        .start(target.path(), NodeType::Stateful)
        .await
        .expect("unable to start the rippled node");
    wait_for_state(&node.rpc_url(), "proposing".into()).await;

    // Create a synthetic node and connect it to rippled.
    let mut synth_node = SyntheticNode::new(&Default::default()).await;
    synth_node
        .connect(node.addr())
        .await
        .expect("unable to connect");

    // Create a payload with a valid key.
    let mut public_key = vec![PUBLIC_KEY_TYPES[0]]; // Place the key type as the first byte.
    public_key.resize(PUBLIC_KEY_SIZE, 0x1); // Append 32 bytes serving as a dummy public key.
    let payload = Payload::TmGetPeerShardInfoV2(TmGetPeerShardInfoV2 {
        peer_chain: vec![TmPublicKey { public_key }],
        relays: 1,
    });

    // Send a message from the synthetic node.
    synth_node
        .unicast(node.addr(), payload)
        .expect("unable to send message");

    // Ensure that the synthetic node receives TmPeerShardInfoV2.
    // This should happen when rippled is configured to use history sharding.
    let check = |m: &BinaryMessage| matches!(&m.payload, Payload::TmPeerShardInfoV2(..));
    assert!(synth_node.expect_message(&check).await);

    // Shutdown.
    synth_node.shut_down().await;
    node.stop().expect("unable to stop the rippled node");
}
