use std::{
    fs, io,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    process::Child,
};

use anyhow::Result;

use crate::setup::{
    config::{NodeConfig, NodeMetaData, RippledConfigFile, RIPPLED_CONFIG, RIPPLED_DIR},
    process,
    process::wait_for_start,
};

pub struct Node {
    /// Fields to be written to the node's configuration file.
    config: NodeConfig,
    /// The node metadata read from Ziggurat's configuration file.
    meta: NodeMetaData,
    /// The process encapsulating the running node.
    process: Option<Child>,
}

impl Node {
    pub fn addr(&self) -> SocketAddr {
        self.config.local_addr
    }

    // TODO change to consume self, it's probably useless now anyway
    pub fn stop(&mut self) -> io::Result<()> {
        if let Some(child) = self.process.take() {
            // Stop node process, and check for crash (needs to happen before cleanup)
            let crashed = process::stop(child);
            self.cleanup()?;

            if let Some(crash_msg) = crashed {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Node exited early, {}", crash_msg),
                ));
            }
        }

        Ok(())
    }

    async fn start_process(&mut self) -> Result<()> {
        // cleanup any previous runs (node.stop won't always be reached e.g. test panics, or SIGINT)
        self.cleanup()?;

        // generate config and start child process
        self.generate_config_file()?;
        let process = process::start(&self.meta, self.config.log_to_stdout);
        wait_for_start(&self.addr()).await;
        self.process = Some(process);

        Ok(())
    }

    fn generate_config_file(&self) -> Result<()> {
        let path = self.config.path.join(RIPPLED_CONFIG);
        let content = RippledConfigFile::generate(&self.config)?;
        fs::write(path, content)?;

        Ok(())
    }

    fn cleanup(&self) -> io::Result<()> {
        self.cleanup_config_file()?;
        self.cleanup_cache()
    }

    fn cleanup_config_file(&self) -> io::Result<()> {
        let path = self.config.path.join(RIPPLED_CONFIG);
        match fs::remove_file(path) {
            // File may not exist, so we suppress the error.
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }

    fn cleanup_cache(&self) -> io::Result<()> {
        let path = self.config.path.join(RIPPLED_DIR);
        if let Err(e) = fs::remove_dir_all(path) {
            // Directory may not exist, so we let that error through
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err(e);
            }
        }

        Ok(())
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        // We should avoid a panic.
        if let Err(e) = self.stop() {
            println!("Failed to stop the node: {}", e);
        }
    }
}

/// Convenience struct to better control configuration/build/start for [Node]
pub struct NodeBuilder {
    config: NodeConfig,
    meta: NodeMetaData,
}

impl NodeBuilder {
    /// Sets up minimal configuration for the node.
    pub fn new(path: PathBuf, ip_addr: IpAddr) -> Result<Self> {
        let config = NodeConfig::new(path, ip_addr);
        let meta = NodeMetaData::new(config.path.clone())?;
        Ok(Self { config, meta })
    }

    /// Sets initial peers for the node.
    pub fn initial_peers(mut self, addrs: Vec<SocketAddr>) -> Self {
        self.config.initial_peers = addrs.into_iter().collect();
        self
    }

    /// Sets validator token to be placed in rippled.cfg.
    /// This will configure the node to run as a validator.
    pub fn validator_token(mut self, token: String) -> Self {
        self.config.validator_token = Some(token);
        self
    }

    /// Sets whether to log the node's output to Ziggurat's output stream.
    pub fn log_to_stdout(mut self, log_to_stdout: bool) -> Self {
        self.config.log_to_stdout = log_to_stdout;
        self
    }

    /// Sets network's id to form an isolated testnet.
    pub fn network_id(mut self, network_id: u32) -> Self {
        self.config.network_id = Some(network_id);
        self
    }

    /// Builds and starts the new node.
    pub async fn build(self) -> Result<Node> {
        let mut node = Node {
            config: self.config,
            meta: self.meta,
            process: None,
        };
        node.start_process().await?;
        Ok(node)
    }
}
