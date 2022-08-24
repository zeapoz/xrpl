use std::{
    collections::HashSet,
    ffi::OsString,
    fmt::Write,
    fs,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

use anyhow::Result;
use serde::Deserialize;

// Ziggurat's configuration directory and file. Caches are written to this directory.
pub const ZIGGURAT_DIR: &str = ".ziggurat";
// Configuration file with paths to start rippled.
pub const ZIGGURAT_CONFIG: &str = "config.toml";

// Rippled's configuration file name.
pub const RIPPLED_CONFIG: &str = "rippled.cfg";
pub const RIPPLED_DIR: &str = "rippled";

// The default port to start a Rippled node on.
pub const DEFAULT_PORT: u16 = 8080;

/// Convenience struct for reading Ziggurat's configuration file.
#[derive(Deserialize)]
struct ConfigFile {
    /// The absolute path of where to run the start command.
    path: PathBuf,
    /// The command to start the node.
    start_command: String,
}

/// The node metadata read from Ziggurat's configuration file.
#[derive(Debug)]
pub struct NodeMetaData {
    /// The absolute path of where to run the start command.
    pub path: PathBuf,
    /// The command to start the node.
    pub start_command: OsString,
    /// The arguments to the start command of the node.
    pub start_args: Vec<OsString>,
}

impl NodeMetaData {
    pub fn new(config_path: PathBuf) -> Result<NodeMetaData> {
        // Read Ziggurat's configuration file.
        let path = config_path.join(ZIGGURAT_CONFIG);
        let config_string = fs::read_to_string(path)?;
        let config_file: ConfigFile = toml::from_str(&config_string)?;

        // Read the args (which includes the start command at index 0).
        let args_from = |command: &str| -> Vec<OsString> {
            command.split_whitespace().map(OsString::from).collect()
        };

        // Separate the start command from the args list.
        let mut start_args = args_from(&config_file.start_command);
        let start_command = start_args.remove(0);

        let joined_path = config_path.join(RIPPLED_CONFIG);
        start_args.push("--conf".into());
        start_args.push(joined_path.into());

        Ok(Self {
            path: config_file.path,
            start_command,
            start_args,
        })
    }
}

/// Fields to be written to the node's configuration file.
#[derive(Debug)]
pub struct NodeConfig {
    /// The path of the cache directory of the node (and Ziggurat); this is `~/.ziggurat`.
    pub path: PathBuf,
    /// The socket address of the node.
    pub local_addr: SocketAddr,
    /// The initial peer set of the node.
    pub initial_peers: HashSet<SocketAddr>,
    /// The initial max number of peer connections to allow.
    pub max_peers: usize,
    /// Toggles node logging to stdout.
    pub log_to_stdout: bool,
    /// Token when run as a validator.
    pub validator_token: Option<String>,
}

impl NodeConfig {
    pub fn new(path: PathBuf, ip_addr: IpAddr) -> Self {
        // Set the port explicitly.
        let local_addr = SocketAddr::new(ip_addr, DEFAULT_PORT);

        Self {
            path,
            local_addr,
            initial_peers: Default::default(),
            max_peers: 50,
            log_to_stdout: false,
            validator_token: None,
        }
    }
}

pub struct RippledConfigFile;

impl RippledConfigFile {
    pub fn generate(config: &NodeConfig) -> Result<String> {
        let mut config_str = String::new();

        // 1. Server

        writeln!(&mut config_str, "[server]")?;
        writeln!(&mut config_str, "port_rpc_admin_local")?;
        writeln!(&mut config_str, "port_peer")?;
        writeln!(&mut config_str)?;

        writeln!(&mut config_str, "[port_rpc_admin_local]")?;
        writeln!(&mut config_str, "port = 5005")?;
        writeln!(&mut config_str, "ip = {}", config.local_addr.ip())?;
        writeln!(&mut config_str, "admin = {}", config.local_addr.ip())?;
        writeln!(&mut config_str, "protocol = http")?;
        writeln!(&mut config_str)?;

        writeln!(&mut config_str, "[port_peer]")?;
        writeln!(&mut config_str, "port = {}", config.local_addr.port())?;
        writeln!(&mut config_str, "ip = {}", config.local_addr.ip())?;
        writeln!(&mut config_str, "protocol = peer")?;
        writeln!(&mut config_str)?;

        if let Some(token) = &config.validator_token {
            writeln!(&mut config_str, "[validator_token]")?;
            writeln!(&mut config_str, "{}", token)?;
            writeln!(&mut config_str)?;
        }

        // 2. Peer protocol
        writeln!(&mut config_str, "[reduce_relay]")?;
        writeln!(&mut config_str, "tx_enable = 1")?;
        writeln!(&mut config_str)?;

        writeln!(&mut config_str, "[ips_fixed]")?;
        for addr in &config.initial_peers {
            writeln!(&mut config_str, "{} {}", addr.ip(), addr.port())?;
        }
        writeln!(&mut config_str)?;

        writeln!(&mut config_str, "[peers_max]")?;
        writeln!(&mut config_str, "{}", config.max_peers)?;
        writeln!(&mut config_str)?;

        writeln!(&mut config_str, "[sntp_servers]")?;
        writeln!(&mut config_str, "time.windows.com")?;
        writeln!(&mut config_str, "time.apple.com")?;
        writeln!(&mut config_str, "time.nist.gov")?;
        writeln!(&mut config_str, "pool.ntp.org")?;
        writeln!(&mut config_str)?;

        // 3. Ripple protocol

        writeln!(&mut config_str, "[validators_file]")?;
        writeln!(&mut config_str, "validators.txt")?;
        writeln!(&mut config_str)?;

        // 4. HTTPS client

        writeln!(&mut config_str, "[ssl_verify]")?;
        writeln!(&mut config_str, "0")?;
        writeln!(&mut config_str)?;

        // 5. Reporting mode

        // 6. Datababase

        writeln!(&mut config_str, "[node_db]")?;
        writeln!(&mut config_str, "type=NuDB")?;
        writeln!(
            &mut config_str,
            "path={}",
            config
                .path
                .join(RIPPLED_DIR)
                .join("db/nudb")
                .to_str()
                .unwrap()
        )?;
        writeln!(&mut config_str, "online_delete=512")?;
        writeln!(&mut config_str, "advisory_delete=0")?;
        writeln!(&mut config_str)?;

        writeln!(&mut config_str, "[database_path]")?;
        writeln!(
            &mut config_str,
            "{}",
            config.path.join(RIPPLED_DIR).join("db").to_str().unwrap()
        )?;
        writeln!(&mut config_str)?;

        // 7. Diagnostics

        writeln!(&mut config_str, "[debug_logfile]")?;
        writeln!(
            &mut config_str,
            "{}",
            config
                .path
                .join(RIPPLED_DIR)
                .join("debug.log")
                .to_str()
                .unwrap()
        )?;
        writeln!(&mut config_str)?;

        // 8. Voting

        // 9. Misc settings

        // 10. Example settings

        Ok(config_str)
    }
}
