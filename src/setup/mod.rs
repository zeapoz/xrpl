//! Utilities for setting up and tearing down node Ripple node instances.

use std::{io, path::PathBuf};

use crate::setup::config::{RIPPLE_WORK_DIR, ZIGGURAT_DIR};

pub mod config;
pub mod node;
pub mod stateful;
pub mod testnet;

pub fn build_ripple_work_path() -> io::Result<PathBuf> {
    Ok(home::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "couldn't find home directory"))?
        .join(ZIGGURAT_DIR)
        .join(RIPPLE_WORK_DIR))
}
