use std::{
    collections::HashSet,
    fs, io,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use anyhow::Result;
use fs_extra::dir::{copy, CopyOptions};
use tokio::io::AsyncWriteExt;

use crate::{
    setup::{
        build_ripple_work_path,
        config::{NodeMetaData, RippledConfigFile, RIPPLED_CONFIG, RIPPLE_SETUP_DIR},
    },
    tools::constants::{CONNECTION_TIMEOUT, DEFAULT_PORT},
};

async fn wait_for_start(addr: SocketAddr) {
    tokio::time::timeout(CONNECTION_TIMEOUT, async move {
        loop {
            if let Ok(mut stream) = tokio::net::TcpStream::connect(addr).await {
                stream.shutdown().await.unwrap();
                break;
            }
        }
    })
    .await
    .unwrap();
}

pub enum ChildExitCode {
    Success,
    ErrorCode(Option<i32>),
}

pub struct NodeBuilder {
    conf: NodeConfig,
    meta: NodeMetaData,
}

impl NodeBuilder {
    /// Creates new [NodeBuilder]. Initial state is taken from `state` path, the node will run in `target` path.
    pub fn new(state: Option<PathBuf>, target: PathBuf) -> Result<Self> {
        if !target.exists() {
            fs::create_dir_all(&target)?;
        }
        let mut copy_options = CopyOptions::new();
        copy_options.content_only = true;
        copy_options.overwrite = true;
        let setup_path = build_ripple_work_path()?.join(RIPPLE_SETUP_DIR);
        copy(&setup_path, &target, &copy_options)?;
        if let Some(source) = state {
            copy(&source, &target, &copy_options)?;
        }
        let conf = NodeConfig::new(target);
        let meta = NodeMetaData::new(conf.path.clone())?;
        Ok(Self { conf, meta })
    }

    /// Crates [Node] according to configuration and starts its process.
    pub async fn start(self, log_to_stdout: bool) -> Result<Node> {
        let content = RippledConfigFile::generate(&self.conf)?;
        let path = self.conf.path.join(RIPPLED_CONFIG);
        fs::write(path, content)?;
        let node = self.start_node(log_to_stdout);
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

    fn start_node(self, log_to_stdout: bool) -> Node {
        let (stdout, stderr) = match log_to_stdout {
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
            meta: self.meta,
            config: self.conf,
        }
    }
}

/// Fields to be written to the node's configuration file.
#[derive(Debug)]
pub struct NodeConfig {
    /// The path of the cache directory of the node.
    pub path: PathBuf,
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
}

impl NodeConfig {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            local_addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, DEFAULT_PORT)),
            initial_peers: Default::default(),
            max_peers: 0,
            validator_token: None,
            network_id: None,
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
    pub fn builder(state: Option<PathBuf>, target: PathBuf) -> NodeBuilder {
        NodeBuilder::new(state, target)
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
