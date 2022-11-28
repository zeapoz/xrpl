use crate::{
    protocol::{
        codecs::message::{BinaryMessage, Payload},
        proto::TransactionStatus::TsCurrent,
    },
    tests::conformance::{perform_testnet_transaction_check, TRANSACTION_BLOB},
};

#[tokio::test]
#[allow(non_snake_case)]
async fn c019_MT_TRANSACTION_node_should_broadcast_transaction_to_all_peers() {
    // ZG-CONFORMANCE-019

    // Ensure that the synthetic node connected to the testnet received the transaction.
    let blob_bytes = hex::decode(TRANSACTION_BLOB).unwrap();
    let check = |m: &BinaryMessage| matches!(&m.payload, Payload::TmTransaction(tm_transaction) if tm_transaction.raw_transaction == blob_bytes && tm_transaction.status == TsCurrent as i32 && tm_transaction.deferred == Some(false));
    perform_testnet_transaction_check(&check).await;
}
