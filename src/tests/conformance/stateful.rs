//! Contains code to start and stop a node with preloaded ledger data.

use tempfile::TempDir;

use crate::{
    setup::node::{Node, NodeType},
    tools::{
        constants::EXPECTED_RESULT_TIMEOUT,
        rpc::{wait_for_account_data, wait_for_state},
    },
};

#[tokio::test]
#[ignore = "convenience test to tinker with a running node for dev purposes"]
async fn should_start_stop_stateful_node() {
    let target = TempDir::new().expect("unable to create TempDir");

    let mut node = Node::builder()
        .start(target.path(), NodeType::Stateful)
        .await
        .expect("unable to start stateful node");
    wait_for_state(&node.rpc_url(), "proposing".into()).await;

    let account_data = wait_for_account_data(
        &node.rpc_url(),
        "rNGknFCRBZguXcPqC63k6xTZnonSe6ZuWt",
        EXPECTED_RESULT_TIMEOUT,
    )
    .await
    .expect("unable to get account data");
    assert_eq!(account_data.result.account_data.balance, "5000000000");

    node.stop().expect("unable to stop stateful node");
}
