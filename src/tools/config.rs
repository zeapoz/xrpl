use std::net::{IpAddr, Ipv4Addr};

use crate::protocol::codecs::binary::Payload;
#[cfg(doc)]
use crate::tools::synth_node::SyntheticNode;

/// Test configuration. Contains setup options for [SyntheticNode] and [pea2pea::Config].
pub struct TestConfig {
    pub synth_node_config: SyntheticNodeTestConfig,
    pub pea2pea_config: pea2pea::Config,
}

impl Default for TestConfig {
    fn default() -> Self {
        let ip_addr = IpAddr::V4(Ipv4Addr::LOCALHOST);
        Self {
            synth_node_config: Default::default(),
            pea2pea_config: pea2pea::Config {
                listener_ip: Some(ip_addr),
                ..Default::default()
            },
        }
    }
}

impl TestConfig {
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
    /// Whether or not to generate new keys for a handshake.
    pub generate_new_keys: bool,
    /// Identification header to be set during a handshake. Either 'User-Agent' or 'Server' depending on connection side.
    pub ident: String,
}

impl Default for SyntheticNodeTestConfig {
    fn default() -> Self {
        Self {
            do_handshake: true,
            initial_message: None,
            generate_new_keys: true,
            ident: "rippled-1.9.1".into(),
        }
    }
}
