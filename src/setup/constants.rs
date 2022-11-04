use std::time::Duration;

/// Ziggurat's configuration directory.
pub const ZIGGURAT_DIR: &str = ".ziggurat";

/// Ziggurat's Ripple's subdir.
pub const RIPPLE_WORK_DIR: &str = "ripple";

/// Initial setup dir for rippled.
pub const RIPPLE_SETUP_DIR: &str = "setup";

/// Configuration file with paths to start rippled.
pub const ZIGGURAT_CONFIG: &str = "config.toml";

/// Validators file name.
pub const VALIDATORS_FILE_NAME: &str = "validators.txt";

/// Directory containing saved ledger and config to be loaded after the start.
pub const STATEFUL_NODES_DIR: &str = "stateful";

/// Number of available stateful nodes
pub const STATEFUL_NODES_COUNT: usize = 3;

/// Validator IP address list
pub const VALIDATOR_IPS: [&str; STATEFUL_NODES_COUNT] = ["127.0.0.1", "127.0.0.2", "127.0.0.3"];

/// Rippled's configuration file name.
pub const RIPPLED_CONFIG: &str = "rippled.cfg";
pub const RIPPLED_DIR: &str = "rippled";

/// Rippled's JSON RPC port
pub const JSON_RPC_PORT: u32 = 5005;

/// The default port to start a Rippled node on.
pub const DEFAULT_PORT: u16 = 8080;

/// [TestNet](crate::setup::testnet::TestNet)'s network id. The number here doesn't have any significance, but cannot be 0 nor 255.
pub const TESTNET_NETWORK_ID: u32 = 239048;

/// Timeout when waiting for [Node](crate::setup::node::Node)'s start.
pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);

/// Timeout when waiting for [TestNet](crate::setup::testnet::TestNet) to start.
pub const TESTNET_READY_TIMEOUT: Duration = Duration::from_secs(60);

/// Rippled cryptographic seed. Used for clustering.
pub const RIPPLED_NODE_SEED: &str = "shH6WH5TB8SychcABC1V5LqqfJR3B";

/// Public key used for handshake in [SyntheticNode](crate::tools::synth_node::SyntheticNode).
pub const SYNTHETIC_NODE_PUBLIC_KEY: &str = "n9K8BSMKPK1PozRhcsoFr2BGTvRTUHra6reFd3n7quG8D2PP6fHG";

/// Private key used for signing messages in [SyntheticNode](crate::tools::synth_node::SyntheticNode).
pub const SYNTHETIC_NODE_PRIVATE_KEY: &str = "pabEKa3CtSNR1VatkL9xqmYm6BpvYyWX3dXS3matz2pKKz37guy";
