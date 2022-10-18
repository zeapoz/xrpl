use std::time::Duration;

/// Timeout when waiting for expected message / node's state.
pub const EXPECTED_RESULT_TIMEOUT: Duration = Duration::from_secs(20);

/// Channel buffer bound for [InnerNode](crate::tools::inner_node::InnerNode) -> [SyntheticNode](crate::tools::synth_node::SyntheticNode) messages.
pub const SYNTH_NODE_QUEUE_DEPTH: usize = 100;

/// Ripple's genesis account. This is an account that holds all XRP when rippled starts from scratch.
pub const GENESIS_ACCOUNT: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";

/// A random but valid account that will be created in tests/setup by sending XRP from the GENESIS_ACCOUNT.
pub const TEST_ACCOUNT: &str = "rNGknFCRBZguXcPqC63k6xTZnonSe6ZuWt";
