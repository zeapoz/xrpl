//! Contains test with peer shard info queries.

use crate::{
    protocol::codecs::binary::{BinaryMessage, Payload},
    tests::conformance::perform_response_test,
};

#[tokio::test]
async fn node_should_query_for_shard_info() {
    // ZG-CONFORMANCE-006
    let response_check =
        |m: &BinaryMessage| matches!(&m.payload, Payload::TmGetPeerShardInfoV2(..));
    perform_response_test(None, &response_check).await;
}
