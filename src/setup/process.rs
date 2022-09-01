//! Set of convenience function to manage rippled process.

use std::{
    net::SocketAddr,
    process::{Child, Command, Stdio},
};

use tokio::io::AsyncWriteExt;

use crate::{setup::config::NodeMetaData, tools::constants::CONNECTION_TIMEOUT, wait_until};

/// Stops the child and collects its exit message.
pub fn stop(mut child: Child) -> Option<String> {
    let message = match child.try_wait().ok()? {
        None => {
            child.kill().ok()?;
            None
        }
        Some(exit_code) if exit_code.success() => {
            Some("but with a \"success\" exit code".to_string())
        }
        Some(exit_code) => Some(format!("crashed with exit code {}", exit_code)),
    };
    child.wait().ok()?;
    message
}

/// Starts a rippled child process according to configuration in [NodeMetaData]
pub fn start(meta: &NodeMetaData, log_to_stdout: bool) -> Child {
    let (stdout, stderr) = match log_to_stdout {
        true => (Stdio::inherit(), Stdio::inherit()),
        false => (Stdio::null(), Stdio::null()),
    };
    Command::new(&meta.start_command)
        .current_dir(&meta.path)
        .args(&meta.start_args)
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
        .expect("node failed to start")
}

/// Waits until given socket starts accepting connections or a timeout elapses.
pub async fn wait_for_start(addr: &SocketAddr) {
    wait_until!(
        CONNECTION_TIMEOUT,
        if let Ok(mut stream) = tokio::net::TcpStream::connect(addr).await {
            stream.shutdown().await.unwrap();
            true
        } else {
            false
        }
    );
}
