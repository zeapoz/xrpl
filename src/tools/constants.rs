use std::time::Duration;

/// Timeout when waiting for [Node](crate::setup::node::Node)'s start.
pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);

/// Timeout for [SyntheticNode](crate::tools::synth_node::SyntheticNode) when waiting for expected message.
pub const EXPECTED_MESSAGE_TIMEOUT: Duration = Duration::from_secs(20);

/// Channel buffer bound for [InnerNode](crate::tools::inner_node::InnerNode) -> [SyntheticNode](crate::tools::synth_node::SyntheticNode) messages.
pub const SYNTH_NODE_QUEUE_DEPTH: usize = 100;

/// [TestNet](crate::setup::testnet::TestNet)'s network id. The number here doesn't have any significance, but cannot be 0 nor 255.
pub const TESTNET_NETWORK_ID: u32 = 239048;

/// The default port to start a Rippled node on.
pub const DEFAULT_PORT: u16 = 8080;
