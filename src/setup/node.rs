use std::{
    collections::HashSet,
    fs, io,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

use anyhow::Result;
use fs_extra::{dir, file};
use tokio::{io::AsyncWriteExt, net::TcpStream, time::Duration};

use crate::{
    setup::{
        build_ripple_work_path,
        config::{
            NodeMetaData, RippledConfigFile, RIPPLED_CONFIG, RIPPLE_SETUP_DIR, STATEFUL_NODES_DIR,
        },
        testnet::get_validator_token,
    },
    tools::constants::{
        CONNECTION_TIMEOUT, DEFAULT_PORT, TESTNET_NETWORK_ID, VALIDATORS_FILE_NAME,
    },
};

async fn wait_for_start(addr: SocketAddr) {
    tokio::time::timeout(CONNECTION_TIMEOUT, async {
        const SLEEP: Duration = Duration::from_millis(10);

        loop {
            if let Ok(mut stream) = TcpStream::connect(addr).await {
                stream.shutdown().await.unwrap();
                break;
            }

            tokio::time::sleep(SLEEP).await;
        }
    })
    .await
    .unwrap();
}

pub enum ChildExitCode {
    Success,
    ErrorCode(Option<i32>),
}

/// Node type is used to select different startup configurations.
pub enum NodeType {
    /// A temporary node used to store ledger data for stateful nodes. Should not be used otherwise.
    Testnet,
    /// A non-validator node without any preloaded data.
    Stateless,
    /// A validator node with a preloaded ledger data.
    Stateful,
}

pub struct NodeBuilder {
    conf: NodeConfig,
    meta: NodeMetaData,
}

impl NodeBuilder {
    /// Creates new [NodeBuilder] which can handle stateless nodes.
    pub fn stateless() -> anyhow::Result<Self> {
        let setup_path = build_ripple_work_path()?.join(RIPPLE_SETUP_DIR);

        let conf = NodeConfig::default();
        let meta = NodeMetaData::new(setup_path)?;
        Ok(Self { conf, meta })
    }

    /// Creates new [NodeBuilder] which can handle stateful nodes.
    pub fn stateful() -> anyhow::Result<Self> {
        Ok(Self::stateless()
            .expect("Failed to create a node builder")
            .network_id(TESTNET_NETWORK_ID)
            .validator_token(get_validator_token(0))
            .add_args(vec![
                "--valid".into(),
                "--quorum".into(),
                "1".into(),
                "--load".into(),
            ]))
    }

    /// Creates [Node] according to configuration and starts its process.
    pub async fn start(&mut self, target: &Path, node_type: NodeType) -> Result<Node> {
        if !target.exists() {
            fs::create_dir_all(&target)?;
        }

        match node_type {
            NodeType::Stateful => {
                let source = get_stateful_node_path()?;

                let mut copy_options = dir::CopyOptions::new();
                copy_options.content_only = true;
                copy_options.overwrite = true;
                dir::copy(&source, &target, &copy_options)?;
            }
            NodeType::Stateless => {
                let setup_path = build_ripple_work_path()?.join(RIPPLE_SETUP_DIR);
                let validators_file_src = setup_path.join(VALIDATORS_FILE_NAME);
                let validators_file_dst = target.join(VALIDATORS_FILE_NAME);

                let copy_options = file::CopyOptions::new();
                file::copy(&validators_file_src, &validators_file_dst, &copy_options)?;

                self.conf.network_id = None;
                self.conf.validator_token = None;
                self.meta = NodeMetaData::new(setup_path)?; // Reset args
            }
            NodeType::Testnet => (),
        }

        let rippled_cfg = RippledConfigFile::generate(&self.conf, target)?;
        let rippled_cfg_path = target.join(RIPPLED_CONFIG);
        fs::write(rippled_cfg_path.clone(), rippled_cfg)?;

        self.meta.start_args.push("--conf".into());
        self.meta.start_args.push(rippled_cfg_path.into());

        let node = self.start_node();
        wait_for_start(node.config.local_addr).await;

        Ok(node)
    }

    /// Sets address to bind to.
    pub fn set_addr(mut self, addr: SocketAddr) -> Self {
        self.conf.local_addr = addr;
        self
    }

    /// Adds arguments to start command.
    pub fn add_args(mut self, args: Vec<String>) -> Self {
        args.into_iter()
            .for_each(|arg| self.meta.start_args.push(arg.into()));
        self
    }

    /// Sets initial peers for the node.
    pub fn initial_peers(mut self, addrs: Vec<SocketAddr>) -> Self {
        self.conf.initial_peers = addrs.into_iter().collect();
        self
    }

    /// Sets validator token to be placed in rippled.cfg.
    /// This will configure the node to run as a validator.
    pub fn validator_token(mut self, token: String) -> Self {
        self.conf.validator_token = Some(token);
        self
    }

    /// Sets network's id to form an isolated testnet.
    pub fn network_id(mut self, network_id: u32) -> Self {
        self.conf.network_id = Some(network_id);
        self
    }

    /// Sets whether to log the node's output to Ziggurat's output stream.
    pub fn log_to_stdout(mut self, log_to_stdout: bool) -> Self {
        self.conf.log_to_stdout = log_to_stdout;
        self
    }

    fn start_node(&self) -> Node {
        let (stdout, stderr) = match self.conf.log_to_stdout {
            true => (Stdio::inherit(), Stdio::inherit()),
            false => (Stdio::null(), Stdio::null()),
        };

        let child = Command::new(&self.meta.start_command)
            .current_dir(&self.meta.path)
            .args(&self.meta.start_args)
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
            .expect("node failed to start");

        Node {
            child,
            meta: self.meta.clone(),
            config: self.conf.clone(),
        }
    }
}

/// Fields to be written to the node's configuration file.
#[derive(Debug, Clone)]
pub struct NodeConfig {
    /// The socket address of the node.
    pub local_addr: SocketAddr,
    /// The initial peer set of the node.
    pub initial_peers: HashSet<SocketAddr>,
    /// The initial max number of peer connections to allow.
    pub max_peers: usize,
    /// Token when run as a validator.
    pub validator_token: Option<String>,
    /// Network's id to form an isolated testnet.
    pub network_id: Option<u32>,
    /// Setting this option to true will enable node logging to stdout.
    pub log_to_stdout: bool,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            local_addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, DEFAULT_PORT)),
            initial_peers: Default::default(),
            max_peers: 0,
            validator_token: None,
            network_id: None,
            log_to_stdout: false,
        }
    }
}

pub struct Node {
    child: Child,
    config: NodeConfig,
    #[allow(dead_code)]
    meta: NodeMetaData,
}

impl Node {
    pub fn builder() -> NodeBuilder {
        NodeBuilder::stateful()
            .map_err(|e| format!("Unable to create builder: {:?}", e))
            .unwrap()
    }

    pub fn stop(&mut self) -> io::Result<ChildExitCode> {
        match self.child.try_wait()? {
            None => self.child.kill()?,
            Some(code) => return Ok(ChildExitCode::ErrorCode(code.code())),
        }
        let exit = self.child.wait()?;

        match exit.code() {
            None => Ok(ChildExitCode::Success),
            Some(exit) if exit == 0 => Ok(ChildExitCode::Success),
            Some(exit) => Ok(ChildExitCode::ErrorCode(Some(exit))),
        }
    }

    pub fn addr(&self) -> SocketAddr {
        self.config.local_addr
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        // We should avoid a panic.
        if let Err(e) = self.stop() {
            eprintln!("Failed to stop the node: {}", e);
        }
    }
}

fn get_stateful_node_path() -> io::Result<PathBuf> {
    // TODO support multiple nodes
    let ziggurat_path = build_ripple_work_path()?;
    Ok(ziggurat_path.join(STATEFUL_NODES_DIR).join("1"))
}
