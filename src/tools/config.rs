use std::net::{IpAddr, Ipv4Addr};

#[cfg(doc)]
use crate::setup::node::Node;
#[cfg(doc)]
use crate::tools::synth_node::SyntheticNode;
use crate::{protocol::codecs::binary::Payload, setup::config::NewNodeConfig};

/// Test configuration. Contains setup options for [SyntheticNode], [Node] and [pea2pea::Config].
pub struct TestConfig {
    pub synth_node_config: SyntheticNodeTestConfig,
    pub real_node_config: NewNodeConfig,
    pub pea2pea_config: pea2pea::Config,
}

impl Default for TestConfig {
    fn default() -> Self {
        let ip_addr = IpAddr::V4(Ipv4Addr::LOCALHOST);
        Self {
            real_node_config: NewNodeConfig::new(
                home::home_dir().expect("Can't find home directory"),
                ip_addr,
            ),
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
}

impl Default for SyntheticNodeTestConfig {
    fn default() -> Self {
        Self {
            do_handshake: true,
            initial_message: None,
        }
    }
}
