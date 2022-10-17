use crate::{
    protocol::{
        codecs::binary::{BinaryMessage, Payload},
        proto::TransactionStatus::TsCurrent,
    },
    setup::{constants::TESTNET_READY_TIMEOUT, testnet::TestNet},
    tools::{
        constants::GENESIS_ACCOUNT,
        rpc::{submit_transaction, wait_for_account_data},
        synth_node::SyntheticNode,
    },
};

#[tokio::test]
#[allow(non_snake_case)]
async fn c019_MT_TRANSACTION_node_should_broadcast_transaction_to_all_peers() {
    // ZG-CONFORMANCE-019
    const NODE_IDS: [usize; 2] = [0, 1];
    // A transaction blob representing a signed transaction. Extracted by executing `tools/transfer.py` and listening with `tcpdump -A -i lo dst port 5005 or src port 5005`.
    const TRANSACTION_BLOB: &str = "12000022000000002400000001201B0000001E61400000012A05F20068400000000000000A73210330E7FC9D56BB25D6893BA3F317AE5BCF33B3291BD63DB32654A313222F7FD020744630440220297389244D36AF12115296F409C446D9A5D808880DC7FF323AA207ED529CE6C802207AAC5D2A96CB102CBDE85D2A4BA814253CA133AC9277041CAE2E1A349FB233FF8114B5F762798A53D543A014CAF8B297CFF8F2F937E883149193D6AED0CBBC25790ADE05D020C9C6D9201DCF";

    // Start a testnet.
    let mut testnet = TestNet::new().unwrap();
    testnet.start().await.unwrap();
    wait_for_account_data(
        &testnet.running[NODE_IDS[0]].rpc_url(),
        GENESIS_ACCOUNT,
        TESTNET_READY_TIMEOUT,
    )
    .await
    .expect("Unable to get the account data.");

    // Start a synthetic node and connect to the second node in the testnet.
    let mut synth_node = SyntheticNode::new(&Default::default()).await;
    synth_node
        .connect(testnet.running[NODE_IDS[1]].addr())
        .await
        .expect("Unable to connect to the second node");

    // Submit a transaction to the first node via RPC.
    let blob_bytes = hex::decode(TRANSACTION_BLOB).unwrap();
    let transaction = submit_transaction(
        &testnet.running[NODE_IDS[0]].rpc_url(),
        TRANSACTION_BLOB.into(),
        false,
    )
    .await
    .expect("Unable to submit the transaction.");
    assert!(transaction.result.accepted);
    assert!(transaction.result.applied);
    assert!(transaction.result.broadcast);

    // Ensure that the synthetic node connected to the second node received the transaction.
    let check = |m: &BinaryMessage| matches!(&m.payload, Payload::TmTransaction(tm_transaction) if tm_transaction.raw_transaction == blob_bytes && tm_transaction.status == TsCurrent as i32 && tm_transaction.deferred == Some(false));
    assert!(synth_node.expect_message(&check).await);

    // Shutdown.
    testnet.stop().await.expect("Unable to stop the testnet.");
    synth_node.shut_down().await;
}
