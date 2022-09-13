//! Contains code to start and stop a node with preloaded ledger data.

use std::time::Duration;

use tempfile::TempDir;

use crate::{setup::stateful::build_stateful_builder, tools::rpc::wait_for_state};

#[tokio::test]
#[ignore = "convenience test to tinker with a running node for dev purposes"]
async fn should_start_stop_stateful_node() {
    let target = TempDir::new().expect("Unable to create TempDir");
    let mut node = build_stateful_builder(target.path().to_path_buf())
        .expect("Unable to get stateful builder")
        .start(false)
        .await
        .expect("Unable to start stateful node");
    wait_for_state("proposing".into()).await;
    tokio::time::sleep(Duration::from_secs(60)).await;
    node.stop().expect("Unable to stop stateful node");
}
