use crate::{
    protocol::{
        codecs::message::{BinaryMessage, Payload},
        proto::TxSetStatus::TsHave,
    },
    tests::conformance::perform_testnet_transaction_check,
};

#[tokio::test]
#[allow(non_snake_case)]
async fn c020_MT_HAVESET_node_should_broadcast_transaction_set_to_all_peers() {
    // ZG-CONFORMANCE-020

    // Ensure that the synthetic node connected to the testnet received mtHAVESET.
    let check = |m: &BinaryMessage| matches!(&m.payload, Payload::TmHaveSet(transaction_set) if transaction_set.status == TsHave as i32 && !transaction_set.hash.is_empty());
    perform_testnet_transaction_check(&check).await;
}
