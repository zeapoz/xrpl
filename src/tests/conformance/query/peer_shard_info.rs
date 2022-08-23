//! Contains test with peer shard info queries.

use crate::{
    protocol::codecs::binary::{BinaryMessage, Payload},
    tests::conformance::perform_response_test,
    tools::config::TestConfig,
};

#[tokio::test]
async fn node_should_query_for_shard_info_after_handshake() {
    // ZG-CONFORMANCE-006
    let response_check =
        |m: &BinaryMessage| matches!(&m.payload, Payload::TmGetPeerShardInfoV2(..));
    perform_response_test(TestConfig::default(), &response_check).await;
}
#[tokio::test]
#[should_panic]
async fn node_should_not_query_for_shard_info_if_no_handshake() {
    // ZG-CONFORMANCE-007
    let response_check =
        |m: &BinaryMessage| matches!(&m.payload, Payload::TmGetPeerShardInfoV2(..));
    perform_response_test(TestConfig::default().with_handshake(false), &response_check).await;
}
