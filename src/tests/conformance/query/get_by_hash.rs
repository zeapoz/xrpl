use tempfile::TempDir;

use crate::{
    protocol::{
        codecs::binary::{BinaryMessage, Payload},
        proto::{
            tm_get_object_by_hash::ObjectType, TmGetObjectByHash, TmHaveTransactions,
            TmIndexedObject, TmTransactions,
        },
    },
    setup::node::{Node, NodeType},
    tools::{
        constants::EXPECTED_RESULT_TIMEOUT,
        rpc::{get_transaction_info, wait_for_account_data, wait_for_state},
        synth_node::SyntheticNode,
    },
};

#[tokio::test]
#[allow(non_snake_case)]
async fn c007_TM_GET_OBJECT_BY_HASH_get_transaction_by_hash() {
    // ZG-CONFORMANCE-007

    // Create stateful node.
    let target = TempDir::new().expect("unable to create TempDir");
    let mut node = Node::builder()
        .start(target.path(), NodeType::Stateful)
        .await
        .expect("unable to start stateful node");

    // Wait for correct state and account data.
    wait_for_state(&node.rpc_url(), "proposing".into()).await;
    let account_data = wait_for_account_data(
        &node.rpc_url(),
        "rNGknFCRBZguXcPqC63k6xTZnonSe6ZuWt",
        EXPECTED_RESULT_TIMEOUT,
    ) // TODO make constant
    .await
    .expect("unable to get account data");

    // Get transaction info by rpc to put in cache.
    let tx = account_data.result.account_data.previous_transaction;
    let _ = get_transaction_info(&node.rpc_url(), tx.clone())
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

    synth_node.shut_down().await;
    node.stop().expect("unable to stop stateful node");
}

#[tokio::test]
#[allow(non_snake_case)]
async fn c008_TM_HAVE_TRANSACTIONS_query_for_transactions_after_have_transactions() {
    // ZG-CONFORMANCE-008

    // Create stateful node.
    let target = TempDir::new().expect("unable to create TempDir");
    let mut node = Node::builder()
        .start(target.path(), NodeType::Stateful)
        .await
        .expect("unable to start stateful node");

    // Wait for correct state and account data.
    // TODO Add enum to represent node's states.
    wait_for_state(&node.rpc_url(), "proposing".into()).await;
    let account_data = wait_for_account_data(
        &node.rpc_url(),
        "rNGknFCRBZguXcPqC63k6xTZnonSe6ZuWt",
        EXPECTED_RESULT_TIMEOUT,
    ) // TODO make constant
    .await
    .expect("unable to get account data");
    // TODO: consider moving transaction hash to some constant.
    let tx = account_data.result.account_data.previous_transaction;

    // Inform about transaction via peer protocol.
    let mut tx_hash = [0u8; 32];
    hex::decode_to_slice(&tx, &mut tx_hash[..]).expect("unable to decode transaction hash");
    let payload = Payload::TmHaveTransactions(TmHaveTransactions {
        hashes: vec![tx_hash.to_vec()],
    });
    let mut synth_node = SyntheticNode::new(&Default::default()).await;
    synth_node
        .connect(node.addr())
        .await
        .expect("unable to connect");
    synth_node
        .unicast(node.addr(), payload)
        .expect("unable to send message");

    // Check for TmGetObjectByHash query.
    let check = |m: &BinaryMessage| {
        matches!(
            &m.payload,
            Payload::TmGetObjectByHash(TmGetObjectByHash { query, objects, .. }) if objects.len() == 1  && *query && objects[0].hash.as_ref().unwrap() == &tx_hash
        )
    };
    assert!(synth_node.expect_message(&check).await);

    synth_node.shut_down().await;
    node.stop().expect("unable to stop stateful node");
}
