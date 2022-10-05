use serde::Deserialize;
use tempfile::TempDir;

use crate::{
    protocol::codecs::binary::{BinaryMessage, Payload},
    setup::node::{Node, NodeType},
    tests::conformance::{PUBLIC_KEY_LENGTH, PUBLIC_KEY_TYPES},
    tools::synth_node::SyntheticNode,
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
    let target = TempDir::new().expect("unable to create TempDir");
    let mut node = Node::builder()
        .start(target.path(), NodeType::Stateless)
        .await
        .expect("unable to start the rippled node");

    // Create a synthetic node and connect it to rippled.
    let mut synth_node = SyntheticNode::new(&Default::default()).await;
    synth_node
        .connect(node.addr())
        .await
        .expect("unable to connect");

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
    assert!(synth_node.expect_message(&check).await);

    // Shutdown.
    synth_node.shut_down().await;
    node.stop().expect("unable to stop the rippled node");
}
