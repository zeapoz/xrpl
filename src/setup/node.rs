use std::{fs, io};

use anyhow::Result;

use crate::setup::config::{NodeConfig, NodeMetaData, RippledConfigFile, RIPPLED_CONFIG};

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
