use crate::{
    protocol::codecs::message::{BinaryMessage, Payload},
    tests::conformance::perform_expected_message_test,
};

#[tokio::test]
#[allow(non_snake_case)]
async fn c021_TM_VALIDATION_node_should_send_validation_after_handshake() {
    // ZG-CONFORMANCE-021

    // Check for a TmValidation message.
    let check = |m: &BinaryMessage| matches!(&m.payload, Payload::TmValidation(..));
    perform_expected_message_test(Default::default(), &check).await;
}
