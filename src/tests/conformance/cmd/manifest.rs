use crate::{
    protocol::codecs::binary::{BinaryMessage, Payload},
    tests::conformance::perform_response_test,
    tools::config::TestConfig,
};

#[should_panic]
#[tokio::test]
#[allow(non_snake_case)]
async fn c018_TM_MANIFEST_node_should_not_send_manifest_if_no_handshake() {
    // ZG-CONFORMANCE-018

    let response_check = |m: &BinaryMessage| matches!(&m.payload, Payload::TmManifests(..));
    perform_response_test(TestConfig::default().with_handshake(false), &response_check).await;
}
