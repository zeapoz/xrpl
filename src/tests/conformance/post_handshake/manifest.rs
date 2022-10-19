use crate::{
    protocol::codecs::binary::{BinaryMessage, Payload},
    tests::conformance::perform_expected_message_test,
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
    perform_expected_message_test(Default::default(), &check).await;
}
