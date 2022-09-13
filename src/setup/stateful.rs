use std::{io, path::PathBuf};

use crate::{
    setup::{
        build_ripple_work_path,
        config::NODE_STATE_DIR,
        node::{Node, NodeBuilder},
        testnet::TestNet,
    },
    tools::constants::TESTNET_NETWORK_ID,
};

pub fn build_stateful_path() -> io::Result<PathBuf> {
    let ziggurat_path = build_ripple_work_path()?;
    Ok(ziggurat_path.join(NODE_STATE_DIR))
}

pub fn build_stateful_builder(target: PathBuf) -> anyhow::Result<NodeBuilder> {
    let source = build_stateful_path()?;
    let testnet = TestNet::new()?;
    Ok(Node::builder(Some(source), target)
        .network_id(TESTNET_NETWORK_ID)
        .validator_token(testnet.setups[0].validator_token.clone())
        .add_args(vec![
            "--valid".into(),
            "--quorum".into(),
            "1".into(),
            "--load".into(),
        ]))
}
