use crate::{
    protocol::codecs::binary::{BinaryMessage, Payload},
    tests::conformance::perform_response_test,
    tools::config::TestConfig,
};

#[tokio::test]
#[allow(non_snake_case)]
async fn c017_TM_MANIFEST_node_should_send_manifest_after_handshake() {
    // ZG-CONFORMANCE-017

    // Check for a TmManifests message.
    let check = |m: &BinaryMessage| {
        if let Payload::TmManifests(manifests) = &m.payload {
            return !manifests.list.is_empty() && !manifests.list[0].stobject.is_empty();
        }
        false
    };
    perform_response_test(Default::default(), &check).await;
}

#[should_panic]
#[tokio::test]
#[allow(non_snake_case)]
async fn c018_TM_MANIFEST_node_should_not_send_manifest_if_no_handshake() {
    // ZG-CONFORMANCE-018

    let response_check = |m: &BinaryMessage| matches!(&m.payload, Payload::TmManifests(..));
    perform_response_test(TestConfig::default().with_handshake(false), &response_check).await;
}
