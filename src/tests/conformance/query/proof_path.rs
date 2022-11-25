use tempfile::TempDir;

use crate::{
    protocol::{
        codecs::message::{BinaryMessage, Payload},
        proto::{TmLedgerMapType, TmProofPathRequest, TmProofPathResponse},
    },
    setup::node::{Node, NodeType},
    tools::{rpc::wait_for_ledger_info, synth_node::SyntheticNode},
};

#[tokio::test]
#[allow(non_snake_case)]
async fn c025_TM_PROOF_PATH_REQUEST_TM_PROOF_PATH_RESPONSE_send_req_expect_rsp() {
    // ZG-CONFORMANCE-025

    // Create a rippled node.
    let target = TempDir::new().expect("unable to create TempDir");
    let mut node = Node::builder()
        .start(target.path(), NodeType::Stateful)
        .await
        .expect("unable to start the rippled node");
    let ledger_info = wait_for_ledger_info(&node.rpc_url())
        .await
        .expect("unable to get ledger info");
    assert!(!ledger_info.result.ledger.account_state.is_empty());

    // Create a synthetic node and connect it to rippled.
    let mut synth_node = SyntheticNode::new(&Default::default()).await;
    synth_node
        .connect(node.addr())
        .await
        .expect("unable to connect");

    // Query for proof_path for every account_state.
    let ledger_hash =
        hex::decode(ledger_info.result.ledger.ledger_hash).expect("unable to decode ledger hash");
    for state in ledger_info.result.ledger.account_state {
        get_proof_path_for_state(&node, &mut synth_node, &ledger_hash, &state).await;
    }

    // Shutdown.
    synth_node.shut_down().await;
    node.stop().expect("unable to stop the rippled node");
}

async fn get_proof_path_for_state(
    node: &Node,
    synth_node: &mut SyntheticNode,
    ledger_hash: &[u8],
    state: &str,
) {
    // Use `state` as the key to query for.
    let key = hex::decode(state).expect("unable to decode the account state");
    let payload = Payload::TmProofPathRequest(TmProofPathRequest {
        key: key.clone(),
        ledger_hash: ledger_hash.to_vec(),
        r#type: TmLedgerMapType::LmAccountState as i32,
    });

    // Send a message from the synthetic node.
    synth_node
        .unicast(node.addr(), payload)
        .expect("unable to send the message");

    // Ensure that the synthetic node receives TmProofPathResponse.
    let check = |m: &BinaryMessage| {
        matches!(&m.payload, Payload::TmProofPathResponse(
        TmProofPathResponse{key: response_key, ledger_hash: response_ledger_hash, path, ..})
            if *response_key == key && response_ledger_hash == ledger_hash && !path.is_empty()
        )
    };
    assert!(synth_node.expect_message(&check).await);
}
