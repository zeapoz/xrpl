use std::{
    fs, io,
    process::{Child, Command, Stdio},
};

use anyhow::Result;

use crate::setup::config::{
    NodeConfig, NodeMetaData, RippledConfigFile, RIPPLED_CONFIG, RIPPLED_DIR,
};

struct Node {
    /// Fields to be written to the node's configuration file.
    config: NodeConfig,
    /// The node metadata read from Ziggurat's configuration file.
    meta: NodeMetaData,
    /// The process encapsulating the running node.
    process: Option<Child>,
}

impl Node {
    fn new() -> Result<Self> {
        let config = NodeConfig::new()?;
        let meta = NodeMetaData::new(config.path.clone())?;

        Ok(Self {
            config,
            meta,
            process: None,
        })
    }

    /// Sets whether to log the node's output to Ziggurat's output stream.
    fn log_to_stdout(&mut self, log_to_stdout: bool) -> &mut Self {
        self.config.log_to_stdout = log_to_stdout;
        self
    }

    fn start(&mut self) -> Result<()> {
        // cleanup any previous runs (node.stop won't always be reached e.g. test panics, or SIGINT)
        self.cleanup()?;

        // TODO: set initial peers/initial actions.
        self.generate_config_file()?;

        let (stdout, stderr) = match self.config.log_to_stdout {
            true => (Stdio::inherit(), Stdio::inherit()),
            false => (Stdio::null(), Stdio::null()),
        };

        let process = Command::new(&self.meta.start_command)
            .current_dir(&self.meta.path)
            .args(&self.meta.start_args)
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
            .expect("node failed to start");

        self.process = Some(process);

        Ok(())
    }

    fn stop(&mut self) -> io::Result<()> {
        if let Some(mut child) = self.process.take() {
            // Stop node process, and check for crash (needs to happen before cleanup)
            let crashed = match child.try_wait()? {
                None => {
                    child.kill()?;
                    None
                }
                Some(exit_code) if exit_code.success() => {
                    Some("but exited successfully somehow".to_string())
                }
                Some(exit_code) => Some(format!("crashed with {}", exit_code)),
            };

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn start_stop_node() {
        let mut node = Node::new().unwrap();

        node.log_to_stdout(true).start().unwrap();

        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        node.stop().unwrap();
    }
}
