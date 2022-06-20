use std::{ffi::OsString, fmt::Write, fs, io, path::PathBuf};

use anyhow::Result;
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
    fn new(config_path: PathBuf) -> Result<NodeMetaData> {
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
    /// The initial max number of peer connections to allow.
    max_peers: usize,
}

impl NodeConfig {
    fn new() -> io::Result<Self> {
        Ok(Self {
            path: home::home_dir()
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "couldn't find home directory")
                })?
                .join(CONFIG),
            max_peers: 50,
        })
    }
}

struct RippledConfigFile;

impl RippledConfigFile {
    fn generate(config: &NodeConfig) -> Result<String> {
        let mut config_str = String::new();

        writeln!(&mut config_str, "[peers_max]")?;
        writeln!(&mut config_str, "{}", config.max_peers)?;

        Ok(config_str)
    }
}

// TODO: split into separate file.
struct Node {
    /// Fields to be written to the node's configuration file.
    config: NodeConfig,
    /// The node metadata read from Ziggurat's configuration file.
    meta: NodeMetaData,
    // process: Option<Child>
}

impl Node {
    fn new() -> Result<Self> {
        let config = NodeConfig::new()?;
        let meta = NodeMetaData::new(config.path.clone())?;

        Ok(Self { config, meta })
    }

    fn start(&self) -> Result<()> {
        // cleanup any previous runs (node.stop won't always be reached e.g. test panics, or SIGINT)
        self.cleanup()?;

        self.generate_config_file()?;

        // TODO: start the node process.

        Ok(())
    }

    fn stop(&self) -> io::Result<()> {
        // TODO: stop the node process and check for crash.

        self.cleanup()
    }

    fn generate_config_file(&self) -> Result<()> {
        let path = self.config.path.join(RIPPLED_CONFIG);
        let content = RippledConfigFile::generate(&self.config)?;

        fs::write(path, content)?;

        Ok(())
    }

    fn cleanup(&self) -> io::Result<()> {
        let path = self.config.path.join(RIPPLED_CONFIG);
        match fs::remove_file(path) {
            // File may not exist, so we supress the error.
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }

        // TODO: determine if any caches need to be cleanup up.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_works() {
        let node = Node::new().unwrap();

        // dbg!(node.config);
        // dbg!(node.meta);

        node.start().unwrap();
    }
}
