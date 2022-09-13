//! Contains code to start and stop a node with preloaded ledger data.

use tempfile::TempDir;

use crate::{
    setup::node::build_stateful_builder,
    tools::rpc::{wait_for_account_data, wait_for_state},
};

#[tokio::test]
#[ignore = "convenience test to tinker with a running node for dev purposes"]
async fn should_start_stop_stateful_node() {
    let target = TempDir::new().expect("unable to create TempDir");
    let mut node = build_stateful_builder(target.path().to_path_buf())
        .expect("unable to get stateful builder")
        .start(false)
        .await
        .expect("unable to start stateful node");
    wait_for_state("proposing".into()).await;
    let account_data = wait_for_account_data("rNGknFCRBZguXcPqC63k6xTZnonSe6ZuWt")
        .await
        .expect("unable to get account data");
    assert_eq!(account_data.result.account_data.balance, "5000000000");
    node.stop().expect("unable to stop stateful node");
}
