//! Contains test with peer shard info queries.

use crate::{
    protocol::codecs::binary::{BinaryMessage, Payload},
    tests::conformance::perform_response_test,
    tools::config::TestConfig,
};

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
