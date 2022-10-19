use crate::{
    protocol::codecs::binary::{BinaryMessage, Payload},
    tests::conformance::perform_expected_message_test,
};

#[tokio::test]
#[allow(non_snake_case)]
async fn c005_TM_GET_PEER_SHARD_INFO_V2_node_should_query_for_shard_info_after_handshake() {
    // ZG-CONFORMANCE-005
    let response_check =
        |m: &BinaryMessage| matches!(&m.payload, Payload::TmGetPeerShardInfoV2(..));
    perform_expected_message_test(Default::default(), &response_check).await;
}
