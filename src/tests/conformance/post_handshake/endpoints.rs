use crate::{
    protocol::codecs::binary::{BinaryMessage, Payload},
    tests::conformance::perform_response_test,
};

#[tokio::test]
#[allow(non_snake_case)]
async fn c018_TM_ENDPOINTS_node_should_send_endpoints_after_handshake() {
    // ZG-CONFORMANCE-018

    // Check for a TmEndpoints message.
    let check = |m: &BinaryMessage| matches!(&m.payload, Payload::TmEndpoints(..));
    perform_response_test(Default::default(), &check).await;
}
