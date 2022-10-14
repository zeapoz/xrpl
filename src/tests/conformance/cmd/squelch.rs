//! Contains test for the squelching functionality.
//!
//! Node can select a subset of its peers to function as the source of proposal and validation messages from a
//! specific validator and suppressing the messages from the rest of its peers by sending a “squelch” message to them.
//!
//! More specifically, the “squelch” message tells a peer to suppress messages originating from a
//! certain validator (identified by a public key) for a given amount of time. After the duration
//! expires, the peer starts relaying messages downstream.
//!
//! Squelching a connected peer which is also a validator is not possible in case when messages
//! originate from that peer.
//!
//!     <- mtPROPOSE_LEDGER (validator1)
//!     <- mtPROPOSE_LEDGER (validator2)
//!     <- mtPROPOSE_LEDGER (validator1)
//!     <- mtPROPOSE_LEDGER (validator2)
//!     -> mtSQUELCH (validator1)
//!     <-
//!     <- mtPROPOSE_LEDGER (validator2)
//!     <-
//!     <- mtPROPOSE_LEDGER (validator2)

use tempfile::TempDir;
use tokio::time::{sleep, timeout, Duration};

use crate::{
    protocol::{
        codecs::binary::{BinaryMessage, Payload},
        proto::{TmProposeSet, TmSquelch},
    },
    setup::{
        constants::STATEFUL_NODES_COUNT,
        node::{Node, NodeType},
    },
    tools::{rpc::wait_for_state, synth_node::SyntheticNode},
};

// Time we shall wait for a TmProposeLedger message.
const WAIT_MSG_TIMEOUT: Duration = Duration::from_secs(7);
const SQUELCH_DURATION_SECS: u32 = 6 * 60; // Six minutes should be an ample time value.
const HANDLE_REMAINING_PROPOSE_MSGS: Duration = Duration::from_millis(300);

#[tokio::test]
#[allow(non_snake_case)]
async fn c009_TM_SQUELCH_cannot_squelch_peer_ledger_proposals() {
    // ZG-CONFORMANCE-009

    // Create a stateful node.
    let target = TempDir::new().expect("Couldn't create a temporary directory");
    let mut node = Node::builder()
        .start(target.path(), NodeType::Stateful)
        .await
        .expect("Unable to start the stateful node");

    // Wait for correct state and account data.
    wait_for_state(&node.rpc_url(), "proposing".into()).await;

    // Connect synth node.
    let mut synth_node = SyntheticNode::new(&Default::default()).await;
    synth_node
        .connect(node.addr())
        .await
        .expect("Unable to connect");

    // Get a validator public key.
    let validator_pub_key: Vec<u8> = wait_for_validator_key_in_propose_msg(&mut synth_node).await;

    // Squelch the validator public key belonging to our only neighbour.
    let msg = Payload::TmSquelch(TmSquelch {
        squelch: true,
        validator_pub_key: validator_pub_key.clone(),
        squelch_duration: Some(SQUELCH_DURATION_SECS),
    });
    synth_node.unicast(node.addr(), msg).unwrap();

    // Ensure all incoming TmProposeLedger messages are handled before the node processes the squelch message.
    sleep(HANDLE_REMAINING_PROPOSE_MSGS).await;

    // Check that the squelch message had no effect and that we will continue to receive TmProposeLedger messages from the node.
    timeout(WAIT_MSG_TIMEOUT, async {
        loop {
            if let (
                _,
                BinaryMessage {
                    payload: Payload::TmProposeLedger(TmProposeSet { node_pub_key, .. }),
                    ..
                },
            ) = synth_node.recv_message().await
            {
                if validator_pub_key == node_pub_key {
                    break;
                }
            }
        }
    })
    .await
    .expect("TmProposeLedger not received in time");

    synth_node.shut_down().await;
    node.stop().expect("Unable to stop the stateful node");
}

#[tokio::test]
#[allow(non_snake_case)]
async fn c016_TM_SQUELCH_squelch_distant_validators() {
    // ZG-CONFORMANCE-016

    const DISTANT_NODES_CNT: usize = STATEFUL_NODES_COUNT - 1;

    // We need to keep alive these temp directories for the whole duration of the test.
    let target_dirs = (0..STATEFUL_NODES_COUNT)
        .map(|_| TempDir::new().expect("Couldn't create a temporary directory"))
        .collect::<Vec<TempDir>>();
    let mut target = target_dirs.iter();

    let mut builder = Node::builder();

    // Create a stateful node that will be our synth node's only peer.
    let mut peer_node = builder
        .start(target.next().unwrap().path(), NodeType::Stateful)
        .await
        .expect("Unable to start the stateful node");

    // Wait for correct state and account data.
    wait_for_state(&peer_node.rpc_url(), "proposing".into()).await;

    // Connect a synth node.
    let mut synth_node = SyntheticNode::new(&Default::default()).await;
    synth_node
        .connect(peer_node.addr())
        .await
        .expect("Unable to connect");

    // Get a validator public key from the only running node.
    let peer_node_validator_key: Vec<u8> =
        wait_for_validator_key_in_propose_msg(&mut synth_node).await;

    // Prepare other nodes which are all mutually connected but not with the synth node.
    let mut peer_addr_list = vec![peer_node.addr()];
    let mut distant_nodes = vec![];
    for _ in 0..DISTANT_NODES_CNT {
        builder = builder
            .log_to_stdout(false) // Explicit configuration until we really need to debug these nodes.
            .initial_peers(peer_addr_list.clone());
        let node = builder
            .start(target.next().unwrap().path(), NodeType::Stateful)
            .await
            .expect("Unable to start the stateful node");

        peer_addr_list.push(node.addr());
        distant_nodes.push(node);
    }

    // Collect validation keys for distant nodes.
    let mut distant_node_keys = vec![];
    timeout(WAIT_MSG_TIMEOUT, async {
        loop {
            let node_pub_key = wait_for_validator_key_in_propose_msg(&mut synth_node).await;
            if node_pub_key == peer_node_validator_key {
                continue;
            }

            if !distant_node_keys.contains(&node_pub_key) {
                distant_node_keys.push(node_pub_key);
                if distant_node_keys.len() == DISTANT_NODES_CNT {
                    break;
                }
            }
        }
    })
    .await
    .expect("TmProposeLedger not received in time");

    // Squelch distant nodes.
    for key in distant_node_keys.iter() {
        let msg = Payload::TmSquelch(TmSquelch {
            squelch: true,
            validator_pub_key: key.clone(),
            squelch_duration: Some(SQUELCH_DURATION_SECS),
        });
        synth_node.unicast(peer_node.addr(), msg).unwrap();
    }

    // Ensure all incoming TmProposeLedger messages are handled before nodes process the squelch message.
    sleep(HANDLE_REMAINING_PROPOSE_MSGS).await;

    // Verify we are not receiving TmProposeLedger messages from distant nodes.
    let expect_timeout_err_for_squelched_nodes = timeout(WAIT_MSG_TIMEOUT, async {
        loop {
            if let (
                _,
                BinaryMessage {
                    payload: Payload::TmProposeLedger(TmProposeSet { node_pub_key, .. }),
                    ..
                },
            ) = synth_node.recv_message().await
            {
                if distant_node_keys.contains(&node_pub_key) {
                    panic!("It shouldn't be possible to receive proposing ledgers from squelched nodes.");
                }
            }
        }
    }).await;

    assert!(expect_timeout_err_for_squelched_nodes.is_err());

    synth_node.shut_down().await;
    peer_node.stop().expect("Unable to stop the stateful node");
    for mut node in distant_nodes {
        node.stop().expect("Unable to stop the stateful node");
    }
}

async fn wait_for_validator_key_in_propose_msg(synth_node: &mut SyntheticNode) -> Vec<u8> {
    timeout(WAIT_MSG_TIMEOUT, async {
        loop {
            if let (
                _,
                BinaryMessage {
                    payload: Payload::TmProposeLedger(TmProposeSet { node_pub_key, .. }),
                    ..
                },
            ) = synth_node.recv_message().await
            {
                return node_pub_key;
            }
        }
    })
    .await
    .expect("TmProposeLedger not received in time")
}
