use std::{ffi::OsString, fmt::Write, fs, io, path::PathBuf};

use anyhow::Result;
use serde::Deserialize;

// Ziggurat's configuration directory and file. Caches are written to this directory.
const CONFIG: &str = ".ziggurat";
const CONFIG_FILE: &str = "config.toml";

// Rippled's configuration file name.
pub const RIPPLED_CONFIG: &str = "rippled.cfg";

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
        let path = config_path.join(CONFIG_FILE);
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
    /// The initial max number of peer connections to allow.
    pub max_peers: usize,

    pub log_to_stdout: bool,
}

impl NodeConfig {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            path: home::home_dir()
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "couldn't find home directory")
                })?
                .join(CONFIG),
            max_peers: 50,
            log_to_stdout: false,
        })
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
        writeln!(&mut config_str, "")?;

        writeln!(&mut config_str, "[port_rpc_admin_local]")?;
        writeln!(&mut config_str, "port = 5005")?;
        writeln!(&mut config_str, "ip = 127.0.0.1")?;
        writeln!(&mut config_str, "admin = 127.0.0.1")?;
        writeln!(&mut config_str, "protocol = http")?;
        writeln!(&mut config_str, "")?;

        writeln!(&mut config_str, "[port_peer]")?;
        writeln!(&mut config_str, "port = 51235")?;
        writeln!(&mut config_str, "ip = 127.0.0.1")?;
        writeln!(&mut config_str, "protocol = peer")?;
        writeln!(&mut config_str, "")?;

        // 2. Peer protocol

        writeln!(&mut config_str, "[peers_max]")?;
        writeln!(&mut config_str, "{}", config.max_peers)?;
        writeln!(&mut config_str, "")?;

        writeln!(&mut config_str, "[sntp_servers]")?;
        writeln!(&mut config_str, "time.windows.com")?;
        writeln!(&mut config_str, "time.apple.com")?;
        writeln!(&mut config_str, "time.nist.gov")?;
        writeln!(&mut config_str, "pool.ntp.org")?;
        writeln!(&mut config_str, "")?;

        // 3. Ripple protocol

        writeln!(&mut config_str, "[validators_file]")?;
        writeln!(&mut config_str, "validators.txt")?;
        writeln!(&mut config_str, "")?;

        // 4. HTTPS client

        writeln!(&mut config_str, "[ssl_verify]")?;
        writeln!(&mut config_str, "0")?;
        writeln!(&mut config_str, "")?;

        // 5. Reporting mode

        // 6. Datababase

        writeln!(&mut config_str, "[node_db]")?;
        writeln!(&mut config_str, "type=NuDB")?;
        writeln!(
            &mut config_str,
            "path={}",
            config.path.join("rippled/db/nudb").to_str().unwrap()
        )?;
        writeln!(&mut config_str, "online_delete=512")?;
        writeln!(&mut config_str, "advisory_delete=0")?;
        writeln!(&mut config_str, "")?;

        writeln!(&mut config_str, "[database_path]")?;
        writeln!(
            &mut config_str,
            "{}",
            config.path.join("rippled/db").to_str().unwrap()
        )?;
        writeln!(&mut config_str, "")?;

        // 7. Diagnostics

        writeln!(&mut config_str, "[debug_logfile]")?;
        writeln!(
            &mut config_str,
            "{}",
            config.path.join("rippled/debug.log").to_str().unwrap()
        )?;
        writeln!(&mut config_str, "")?;

        // 8. Voting

        // 9. Misc settings

        // 10. Example settings

        Ok(config_str)
    }
}
