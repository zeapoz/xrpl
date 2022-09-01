//! Contains code to start and stop a node with preloaded ledger data.
//!
use std::time::Duration;

use crate::setup::node::Node;

#[tokio::test]
#[ignore = "convenience test to tinker with a running node for dev purposes"]
async fn should_start_stop_stateful_node() {
    let mut node = Node::stateful().await.unwrap();
    tokio::time::sleep(Duration::from_secs(10)).await;
    node.stop().unwrap();
}
