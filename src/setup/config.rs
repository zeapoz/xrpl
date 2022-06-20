use std::{ffi::OsString, fs, io, path::PathBuf};

use serde::Deserialize;

// Ziggurat's configuration directory and file. Caches are written to this directory.
const CONFIG: &str = ".ziggurat";
const CONFIG_FILE: &str = "config.toml";

// Ripple's configuration file name.
const RIPPLED_CONFIG: &str = "rippled.cfg";

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
struct NodeMetaData {
    /// The absolute path of where to run the start command.
    path: PathBuf,
    /// The command to start the node.
    start_command: OsString,
    /// The arguments to the start command of the node.
    start_args: Vec<OsString>,
}

impl NodeMetaData {
    fn new(config_path: PathBuf) -> io::Result<NodeMetaData> {
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
        start_args.push(format!("--conf {}", joined_path.to_str().unwrap()).into());

        Ok(Self {
            path: config_file.path,
            start_command,
            start_args,
        })
    }
}

/// Fields to be written to the node's configuration file.
#[derive(Debug)]
struct NodeConfig {
    /// The path of the cache directory of the node (and Ziggurat); this is `~/.ziggurat`.
    path: PathBuf,
}

impl NodeConfig {
    fn new() -> io::Result<Self> {
        Ok(Self {
            path: home::home_dir()
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "couldn't find home directory")
                })?
                .join(CONFIG),
        })
    }
}

struct Node {
    /// Fields to be written to the node's configuration file.
    config: NodeConfig,
    /// The node metadata read from Ziggurat's configuration file.
    meta: NodeMetaData,
    // process: Option<Child>
}

impl Node {
    fn new() -> io::Result<Self> {
        let config = NodeConfig::new()?;
        let meta = NodeMetaData::new(config.path.clone())?;

        Ok(Self { config, meta })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_works() {
        let node = Node::new().unwrap();

        dbg!(node.config);
        dbg!(node.meta);
    }
}
