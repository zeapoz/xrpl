use std::io;

#[cfg(doc)]
use crate::setup::node::Node;
#[cfg(doc)]
use crate::tools::synth_node::SyntheticNode;
use crate::{protocol::codecs::binary::Payload, setup::config::NodeConfig};

/// Test configuration. Contains setup options for [SyntheticNode], [Node] and [pea2pea::Config].
pub struct TestConfig {
    pub synth_node_config: SyntheticNodeTestConfig,
    pub real_node_config: NodeConfig,
    pub pea2pea_config: pea2pea::Config,
}

impl TestConfig {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            real_node_config: NodeConfig::new()?,
            synth_node_config: Default::default(),
            pea2pea_config: pea2pea::Config {
                listener_ip: Some("127.0.0.1".parse().unwrap()),
                ..Default::default()
            },
        })
    }

    pub fn with_handshake(mut self, handshake: bool) -> Self {
        self.synth_node_config.do_handshake = handshake;
        self
    }

    pub fn with_initial_message(mut self, payload: Payload) -> Self {
        self.synth_node_config.initial_message = Some(payload);
        self
    }
}

pub struct SyntheticNodeTestConfig {
    /// Whether or not to call `enable_handshake` when creating a new node
    pub do_handshake: bool,
    /// Initial message to be sent to real node
    pub initial_message: Option<Payload>,
}

impl Default for SyntheticNodeTestConfig {
    fn default() -> Self {
        Self {
            do_handshake: true,
            initial_message: None,
        }
    }
}
