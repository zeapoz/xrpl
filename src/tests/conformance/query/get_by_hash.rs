use tempfile::TempDir;

use crate::{
    protocol::{
        codecs::binary::{BinaryMessage, Payload},
        proto::{
            tm_get_object_by_hash::ObjectType, TmGetObjectByHash, TmIndexedObject, TmTransactions,
        },
    },
    setup::node::build_stateful_builder,
    tools::{
        rpc::{get_transaction_info, wait_for_account_data, wait_for_state},
        synth_node::SyntheticNode,
    },
};

#[tokio::test]
async fn should_get_transaction_by_hash() {
    // ZG-CONFORMANCE-008
    // Create stateful node.
    let target = TempDir::new().expect("unable to create TempDir");
    let mut node = build_stateful_builder(target.path().to_path_buf())
        .expect("unable to get stateful builder")
        .start(false)
        .await
        .expect("unable to start stateful node");

    // Wait for correct state and account data.
    wait_for_state("proposing".into()).await;
    let account_data = wait_for_account_data("rNGknFCRBZguXcPqC63k6xTZnonSe6ZuWt") // TODO make constant
        .await
        .expect("unable to get account data");

    // Get transaction info by rpc to put in cache.
    let tx = account_data.result.account_data.previous_transaction;
    let _ = get_transaction_info(tx.clone())
        .await
        .expect("unable to get transaction info");

    // Query transaction via peer protocol.
    let mut tx_hash = [0u8; 32];
    hex::decode_to_slice(&tx, &mut tx_hash as &mut [u8])
        .expect("unable to decode transaction hash");
    let payload = Payload::TmGetObjectByHash(TmGetObjectByHash {
        r#type: ObjectType::OtTransactions as i32,
        query: true,
        seq: Some(1),
        ledger_hash: None,
        fat: None,
        objects: vec![TmIndexedObject {
            hash: Some(tx_hash.into()),
            node_id: None,
            index: None,
            data: None,
            ledger_seq: None,
        }],
    });
    let mut synth_node = SyntheticNode::new(&Default::default()).await;
    synth_node
        .connect(node.addr())
        .await
        .expect("unable to connect");
    synth_node
        .unicast(node.addr(), payload)
        .expect("unable to send message");

    // Check for TmTransactions response with 1 transaction.
    let check = |m: &BinaryMessage| {
        matches!(
            &m.payload,
            Payload::TmTransactions(TmTransactions {transactions}) if transactions.len() == 1
        )
    };
    assert!(synth_node.expect_message(&check).await);
    node.stop().expect("unable to stop stateful node");
}
