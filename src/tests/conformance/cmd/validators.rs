use serde::Deserialize;

use crate::{
    protocol::codecs::binary::{BinaryMessage, Payload},
    tests::conformance::{perform_expected_message_test, PUBLIC_KEY_LENGTH, PUBLIC_KEY_TYPES},
};

#[derive(Deserialize)]
struct ValidatorList {
    validators: Vec<Validator>,
}

#[derive(Deserialize)]
struct Validator {
    validation_public_key: String,
    manifest: String,
}

#[tokio::test]
#[allow(non_snake_case)]
async fn c015_TM_VALIDATOR_LIST_COLLECTION_node_should_send_validator_list() {
    // ZG-CONFORMANCE-015

    // Check for a TmValidatorListCollection message.
    let check = |m: &BinaryMessage| {
        if let Payload::TmValidatorListCollection(validator_list_collection) = &m.payload {
            if let Some(blob_info) = validator_list_collection.blobs.first() {
                let decoded_blob =
                    base64::decode(&blob_info.blob).expect("unable to decode a blob");
                let text = String::from_utf8(decoded_blob)
                    .expect("unable to convert decoded blob bytes to a string");
                let validator_list = serde_json::from_str::<ValidatorList>(&text)
                    .expect("unable to deserialize a validator list");
                if validator_list.validators.is_empty() {
                    return false;
                }
                for validator in &validator_list.validators {
                    let key = hex::decode(&validator.validation_public_key)
                        .expect("unable to decode a public key");
                    if key.len() != PUBLIC_KEY_LENGTH {
                        panic!("invalid public key length: {}", key.len());
                    }
                    if !PUBLIC_KEY_TYPES.contains(&key[0]) {
                        panic!("invalid public key type: {}", key[0]);
                    }
                    if validator.manifest.is_empty() {
                        panic!("empty manifest");
                    }
                }
                return true;
            }
        }
        false
    };
    perform_expected_message_test(Default::default(), &check).await;
}
