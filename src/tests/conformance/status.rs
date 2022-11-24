use tempfile::TempDir;

use crate::{
    protocol::{
        codecs::message::{BinaryMessage, Payload},
        proto::TmStatusChange,
    },
    setup::node::{Node, NodeType},
    tools::{rpc::wait_for_ledger_info, synth_node::SyntheticNode},
};

#[tokio::test]
#[allow(non_snake_case)]
async fn c010_TM_STATUS_CHANGE_node_should_send_ledger_information_using_status_change() {
    let target = TempDir::new().expect("unable to create TempDir");

    // Create a stateful node.
    let mut node = Node::builder()
        .start(target.path(), NodeType::Stateful)
        .await
        .expect("unable to start stateful node");

    // Connect synth node.
    let mut sn = SyntheticNode::new(&Default::default()).await;
    sn.connect(node.addr()).await.unwrap();

    // Get ledger information via RPC.
    let info = wait_for_ledger_info(&node.rpc_url())
        .await
        .expect("no ledger info within the specified time limit");
    let rpc_ledger_index = info
        .result
        .ledger
        .ledger_index
        .parse::<u32>()
        .unwrap_or_else(|_| {
            panic!(
                "unable to parse ledger_index from response: '{}'",
                info.result.ledger.ledger_index
            );
        });
    let mut rpc_ledger_hash = [0u8; 32];
    hex::decode_to_slice(&info.result.ledger.ledger_hash, &mut rpc_ledger_hash[..])
        .expect("unable to decode ledger hash");

    // Wait for TmStatusChange message.
    let check = |m: &BinaryMessage| {
        matches!(
            &m.payload,
            Payload::TmStatusChange(TmStatusChange {
                ledger_seq: Some(ledger_seq),
                ledger_hash: Some(ledger_hash),
                ..
            })
            if ledger_seq == &rpc_ledger_index && ledger_hash.as_slice() == rpc_ledger_hash
        )
    };
    assert!(sn.expect_message(&check).await);

    // Cleanup.
    sn.shut_down().await;
    node.stop().expect("unable to stop stateful node");
}
