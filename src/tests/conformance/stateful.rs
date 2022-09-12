//! Contains code to start and stop a node with preloaded ledger data.
use std::{io, path::PathBuf};

use tempfile::TempDir;

use crate::{
    setup::{build_ripple_work_path, config::NODE_STATE_DIR, node::Node, testnet::TestNet},
    tools::{constants::TESTNET_NETWORK_ID, rpc::wait_for_state},
};

#[tokio::test]
#[ignore = "convenience test to tinker with a running node for dev purposes"]
async fn should_start_stop_stateful_node() {
    let source = build_stateful_path().expect("Unable to build stateful path");
    let target = TempDir::new().expect("Unable to create TempDir");
    let testnet = TestNet::new().expect("Unable to create testnet");
    let mut node = Node::builder(Some(source), target.path().to_path_buf())
        .network_id(TESTNET_NETWORK_ID)
        .validator_token(testnet.setups[0].validator_token.clone())
        .add_args(vec![
            "--valid".into(),
            "--quorum".into(),
            "1".into(),
            "--load".into(),
        ])
        .start(false)
        .await
        .expect("Unable to start stateful node");
    wait_for_state("proposing".into()).await;
    node.stop().expect("Unable to stop stateful node");
}

fn build_stateful_path() -> io::Result<PathBuf> {
    let ziggurat_path = build_ripple_work_path()?;
    Ok(ziggurat_path.join(NODE_STATE_DIR))
}
