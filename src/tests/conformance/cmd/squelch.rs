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
    setup::node::{Node, NodeType},
    tools::{rpc::wait_for_state, synth_node::SyntheticNode},
};

// Time we shall wait for a TmProposeLedger message.
const WAIT_MSG_TIMEOUT: Duration = Duration::from_secs(5);
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
