//! Contains test with ledger queries.
//! ZG-CONFORMANCE-004
//! Queries and expected replies:
//!
//!     - mtGET_LEDGER -> mtLEDGER_DATA

use crate::{
    protocol::{
        codecs::binary::{BinaryMessage, Payload},
        proto::{TmGetLedger, TmLedgerInfoType, TmLedgerType},
    },
    setup::node::Node,
    tools::synth_node::SyntheticNode,
};

#[tokio::test]
async fn should_respond_with_ledger_data_for_basic_info() {
    let payload = Payload::TmGetLedger(TmGetLedger {
        itype: TmLedgerInfoType::LiBase as i32,
        ltype: Some(TmLedgerType::LtClosed as i32),
        ledger_hash: None,
        ledger_seq: None,
        node_i_ds: vec![],
        request_cookie: None,
        query_type: None,
        query_depth: None,
    });
    check_for_ledger_data_response(payload).await;
}

#[tokio::test]
async fn should_respond_with_ledger_data_for_account_state_info() {
    let payload = Payload::TmGetLedger(TmGetLedger {
        itype: TmLedgerInfoType::LiAsNode as i32,
        ltype: Some(TmLedgerType::LtClosed as i32),
        ledger_hash: None,
        ledger_seq: None,
        // Anything other than itype = TmLedgerInfoType::LiBase above requires list of nodes' ids.
        // Here, only one node with id build from 0s to ease deserialize inside ripple.
        node_i_ds: vec![vec![0u8; 33]],
        request_cookie: None,
        query_type: None,
        query_depth: None,
    });
    check_for_ledger_data_response(payload).await;
}

async fn check_for_ledger_data_response(payload: Payload) {
    // Start Ripple node
    let mut node = Node::start_with_peers(vec![]).await.unwrap();

    // Start synth node and connect to Ripple
    let mut synth_node = SyntheticNode::start().await.unwrap();
    synth_node.connect(node.addr()).await.unwrap();

    // Send message
    synth_node.unicast(node.addr(), payload).unwrap();

    // Wait for 'mtLEDGER_DATA' response
    let check = |m: &BinaryMessage| matches!(&m.payload, Payload::TmLedgerData(..));
    assert!(synth_node.expect_message(&check).await);

    // Shutdown both nodes
    synth_node.shut_down().await;
    node.stop().unwrap();
}
