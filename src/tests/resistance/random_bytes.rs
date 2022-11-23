use std::time::Duration;

use tempfile::TempDir;
use tokio::time::sleep;

use crate::{
    fuzzing::{random_bytes, seeded_rng},
    setup::node::{Node, NodeType},
    tools::synth_node::SyntheticNode,
};

const ITERATIONS: usize = 20;
const DISCONNECT_TIMEOUT: Duration = Duration::from_millis(50);

#[tokio::test]
async fn r002_node_must_disconnect_when_receiving_random_bytes() {
    // ZG-RESISTANCE-002

    let mut rng = seeded_rng();
    let payloads = random_bytes(&mut rng, ITERATIONS);

    let target = TempDir::new().expect("couldn't create a temporary directory");

    let mut node = Node::builder()
        .start(target.path(), NodeType::Stateless)
        .await
        .expect("unable to start the node");

    for payload in payloads {
        let synth_node = SyntheticNode::new(&Default::default()).await;
        synth_node.connect(node.addr()).await.unwrap();
        synth_node.unicast_bytes(node.addr(), payload).unwrap();

        // Wait for the node to receive the message and disconnect us.
        sleep(DISCONNECT_TIMEOUT).await;

        // Ensure that the node has disconnected.
        assert!(!synth_node.is_connected_ip(node.addr().ip()));
        synth_node.shut_down().await;
    }

    node.stop().unwrap();
}
