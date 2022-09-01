use std::{
    fs, io,
    net::{IpAddr, SocketAddr, SocketAddrV4},
    path::PathBuf,
    process::Child,
};

use anyhow::Result;
use fs_extra::dir::{copy, CopyOptions};
use tempfile::TempDir;

use crate::{
    setup::{
        config::{
            NewNodeConfig, NodeMetaData, RippledConfigFile, RIPPLED_CONFIG, RIPPLED_DIR,
            ZIGGURAT_DIR,
        },
        process,
        process::wait_for_start,
    },
    tools::constants::{JSON_RPC_PORT, STATEFUL_DIR, STATEFUL_IP},
};

pub enum NodeConfig {
    Stateless(NewNodeConfig),
    Stateful(TempDir),
}

pub struct Node {
    /// Fields to be written to the node's configuration file.
    config: NodeConfig,
    /// The node metadata read from Ziggurat's configuration file.
    meta: NodeMetaData,
    /// The process encapsulating the running node.
    process: Option<Child>,
}

impl Node {
    /// Creates a new running node with the config and database loaded from the predefined state directory.
    /// Before the start, content is copied to a new temporary directory and loaded from there.
    /// This is done to avoid `poisoning` the predefined state directory.
    pub async fn stateful() -> Result<Node> {
        let from = build_stateful_path()?;
        let temp_dir = TempDir::new()?;
        copy(from, &temp_dir, &CopyOptions::new())?;
        let mut meta = NodeMetaData::new(temp_dir.path().to_path_buf().join(STATEFUL_DIR))?;
        meta.start_args.append(&mut vec![
            "--valid".into(),
            "--quorum".into(),
            "1".into(),
            "--load".into(),
        ]);
        let process = process::start(&meta, true);
        wait_for_start(&SocketAddr::V4(SocketAddrV4::new(
            STATEFUL_IP,
            JSON_RPC_PORT,
        )))
        .await;
        Ok(Node {
            config: NodeConfig::Stateful(temp_dir),
            meta,
            process: Some(process),
        })
    }

    pub fn addr(&self) -> SocketAddr {
        // TODO move to NodeConfig
        match &self.config {
            NodeConfig::Stateless(config) => config.local_addr,
            NodeConfig::Stateful(_) => {
                SocketAddr::V4(SocketAddrV4::new(STATEFUL_IP, JSON_RPC_PORT))
            }
        }
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
        let log_to_stdout = match &self.config {
            // TODO move to NodeConfig
            NodeConfig::Stateless(config) => config.log_to_stdout,
            NodeConfig::Stateful(_) => true, // For now, stateful node logs to stdout.
        };
        let process = process::start(&self.meta, log_to_stdout);
        wait_for_start(&self.addr()).await;
        self.process = Some(process);

        Ok(())
    }

    fn generate_config_file(&self) -> Result<()> {
        if let NodeConfig::Stateless(config) = &self.config {
            // TODO move to NodeConfig
            let path = config.path.join(RIPPLED_CONFIG);
            let content = RippledConfigFile::generate(config)?;
            fs::write(path, content)?;
        }
        Ok(())
    }

    fn cleanup(&self) -> io::Result<()> {
        self.cleanup_config_file()?;
        self.cleanup_cache()
    }

    fn cleanup_config_file(&self) -> io::Result<()> {
        let path = match &self.config {
            // TODO move to NodeConfig
            NodeConfig::Stateless(config) => config.path.clone(),
            NodeConfig::Stateful(path) => path.path().to_path_buf(),
        };
        let path = path.join(RIPPLED_CONFIG);
        match fs::remove_file(path) {
            // File may not exist, so we suppress the error.
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }

    fn cleanup_cache(&self) -> io::Result<()> {
        let path = match &self.config {
            // TODO move to NodeConfig
            NodeConfig::Stateless(config) => config.path.clone(),
            NodeConfig::Stateful(path) => path.path().to_path_buf(),
        };
        let path = path.join(RIPPLED_DIR);
        if let Err(e) = fs::remove_dir_all(path) {
            // Directory may not exist, so we let that error through
            if e.kind() != io::ErrorKind::NotFound {
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

fn build_stateful_path() -> io::Result<PathBuf> {
    Ok(home::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "couldn't find home directory"))?
        .join(ZIGGURAT_DIR)
        .join(STATEFUL_DIR))
}

/// Convenience struct to better control configuration/build/start for [Node]
pub struct NodeBuilder {
    config: NewNodeConfig,
    meta: NodeMetaData,
}

impl NodeBuilder {
    /// Sets up minimal configuration for the node.
    pub fn new(path: PathBuf, ip_addr: IpAddr) -> Result<Self> {
        let config = NewNodeConfig::new(path, ip_addr);
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
            config: NodeConfig::Stateless(self.config),
            meta: self.meta,
            process: None,
        };
        node.start_process().await?;
        Ok(node)
    }
}
