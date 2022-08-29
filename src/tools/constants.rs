use std::time::Duration;

/// Timeout for [SyntheticNode] when waiting for expected message.
pub const EXPECTED_MESSAGE_TIMEOUT: Duration = Duration::from_secs(20);

/// Channel buffer bound for [InnerNode] -> [SyntheticNode] messages.
pub const SYNTH_NODE_QUEUE_DEPTH: usize = 100;

/// [TestNet]'s network id. The number here doesn't have any significance, but cannot be 0 nor 255.
pub const TESTNET_NETWORK_ID: u32 = 239048;
