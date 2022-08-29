//! Utilities for setting up a testnet consisting of 3 nodes.

use std::{
    fmt,
    fmt::Write,
    fs, io,
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
};

use crate::{
    setup::{
        config::{DEFAULT_PORT, ZIGGURAT_CONFIG, ZIGGURAT_DIR},
        node::{Node, NodeBuilder},
    },
    tools::constants::TESTNET_NETWORK_ID,
};

/// Testnet's directory for nodes' configs.
const TESTNET_DIR: &str = "testnet";

/// Validators file name.
const VALIDATORS_FILE_NAME: &str = "validators.txt";

const VALIDATOR_KEYS: [&str; 3] = [
    "nHUSqn9qjEF7JJkVqvY7BFLMKdqP5KLLEjo5oB4QH43ADDndRawB",
    "nHUEsvSFTf1Snr7ZUdLxjcMW6PKcMrwwXCGZBg6xb1ePG8R4C3TS",
    "nHUuYdS49cPfRmCXPTwu7MVVFZFFmfG7y5sRttirVMhwuD7xStQp",
];

/// A struct to conveniently start and stop a small testnet.
pub struct TestNet {
    // Setup information for each node. Used for writing configuration.
    setups: [NodeSetup; VALIDATOR_KEYS.len()],
    // Running nodes. Used to stop the testnet.
    running: Vec<Node>,
    // Sets whether to log the node's output to Ziggurat's output stream.
    use_stdout: bool,
    // Path under which all nodes will be built
    path: PathBuf,
}

impl TestNet {
    /// Creates a new TestNet (without starting it).
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            setups: [
                NodeSetup::new(
                    "127.0.0.1".parse().unwrap(),
                    VALIDATOR_KEYS[0].into(),
                    include_str!("token1.txt").into(),
                ),
                NodeSetup::new(
                    "127.0.0.2".parse().unwrap(),
                    VALIDATOR_KEYS[1].into(),
                    include_str!("token2.txt").into(),
                ),
                NodeSetup::new(
                    "127.0.0.3".parse().unwrap(),
                    VALIDATOR_KEYS[2].into(),
                    include_str!("token3.txt").into(),
                ),
            ],
            running: vec![],
            use_stdout: false,
            path: build_testnet_path()?,
        })
    }

    /// Starts a testnet.
    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.cleanup().await?;
        let validators_contents = self.build_validators_file_contents().await?;
        for (i, setup) in self.setups.iter().enumerate() {
            let node = self
                .start_node(&(i + 1).to_string(), setup, &validators_contents)
                .await?;
            self.running.push(node);
        }
        Ok(())
    }

    /// Stops the testnet.
    pub async fn stop(mut self) -> anyhow::Result<()> {
        self.running.iter_mut().for_each(|node| {
            if let Err(e) = node.stop() {
                eprintln!("Unable to stop node: {:?}", e);
            }
        });
        self.cleanup().await?;
        Ok(())
    }

    // Creates `validators.txt` file with keys of all nodes.
    async fn build_validators_file_contents(&self) -> Result<String, fmt::Error> {
        let mut config_str = String::new();
        writeln!(&mut config_str, "[validators]")?;
        for n in &self.setups {
            writeln!(&mut config_str, "{}", n.validator_key)?;
        }
        Ok(config_str)
    }

    // Removes ~/.ziggurat/testnet directory
    async fn cleanup(&self) -> io::Result<()> {
        if let Err(e) = fs::remove_dir_all(&self.path) {
            // Directory may not exist, so we let that error through
            if e.kind() != io::ErrorKind::NotFound {
                return Err(e);
            }
        }
        Ok(())
    }

    // Starts a node in the testnet. `suffix` is used to determine a name for the node's subdirectory.
    async fn start_node(
        &self,
        suffix: &str,
        setup: &NodeSetup,
        validators_contents: &str,
    ) -> anyhow::Result<Node> {
        let path = self.path.join(suffix);
        fs::create_dir_all(&path)?;
        write_validators_file(&path, validators_contents).await?;
        copy_config_file(&path).await?;
        NodeBuilder::new(path, setup.ip)?
            .initial_peers(self.collect_other_peers(setup))
            .log_to_stdout(self.use_stdout)
            .validator_token(setup.validator_token.clone())
            .network_id(TESTNET_NETWORK_ID)
            .build()
            .await
    }

    // Builds a list of peers for the node. Each node has two peers (the other nodes in the testnet).
    fn collect_other_peers(&self, setup: &NodeSetup) -> Vec<SocketAddr> {
        self.setups
            .iter()
            .filter_map(|peer| {
                if peer.ip != setup.ip {
                    Some(SocketAddr::new(peer.ip, DEFAULT_PORT))
                } else {
                    None
                }
            })
            .collect()
    }
}
// Copies config.toml to a node's subdirectory.
async fn copy_config_file(target: &Path) -> io::Result<u64> {
    let source = home::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "couldn't find home directory"))?
        .join(ZIGGURAT_DIR) // TODO extract `home` + `.ziggurat` to separate function
        .join(ZIGGURAT_CONFIG);
    fs::copy(source, target.join(ZIGGURAT_CONFIG))
}

// Saves `validators.txt` file in a node's subdirectory.
async fn write_validators_file(path: &Path, contents: &str) -> io::Result<()> {
    let path = path.join(VALIDATORS_FILE_NAME);
    fs::write(path, contents)
}

// Convenience function to build testnet's path.
fn build_testnet_path() -> io::Result<PathBuf> {
    Ok(home::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "couldn't find home directory"))?
        .join(ZIGGURAT_DIR)
        .join(TESTNET_DIR))
}

// Describes each node's setup.
struct NodeSetup {
    // The node's ip address.
    ip: IpAddr,
    // The node's validator key to be put in the validators.txt file.
    validator_key: String,
    // The node's validator token to be put in the rippled.cfg file.
    validator_token: String,
}

impl NodeSetup {
    fn new(ip: IpAddr, validator_key: String, validator_token: String) -> Self {
        Self {
            ip,
            validator_key,
            validator_token,
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use crate::setup::testnet::TestNet;

    #[ignore = "convenience test to tinker with a running nodes for dev purposes"]
    #[tokio::test]
    async fn should_start_stop_testnet() {
        let mut testnet = TestNet::new().unwrap();
        testnet.use_stdout = true;
        testnet.start().await.unwrap();
        // TODO wait for nodes to start and verify state. At the moment the test is successful it it doesn't panic.
        tokio::time::sleep(Duration::from_secs(10)).await;
        testnet.stop().await.unwrap();
    }
}
