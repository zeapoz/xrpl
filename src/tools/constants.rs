use std::time::Duration;

/// Timeout when waiting for expected message / node's state.
pub const EXPECTED_RESULT_TIMEOUT: Duration = Duration::from_secs(20);

/// Channel buffer bound for [InnerNode](crate::tools::inner_node::InnerNode) -> [SyntheticNode](crate::tools::synth_node::SyntheticNode) messages.
pub const SYNTH_NODE_QUEUE_DEPTH: usize = 100;
