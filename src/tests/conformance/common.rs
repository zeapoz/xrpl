use crate::{setup::node::Node, tools::synth_node::SyntheticNode};

pub async fn start_synth_node() -> SyntheticNode {
    let node_config = pea2pea::Config {
        listener_ip: Some("127.0.0.1".parse().unwrap()),
        ..Default::default()
    };
    SyntheticNode::new(node_config).await
}

pub async fn start_ripple_node() -> Node {
    let mut node = Node::new().unwrap();
    node.log_to_stdout(false).start().await.unwrap();
    node
}
