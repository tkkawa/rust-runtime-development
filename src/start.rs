use std::path::PathBuf;

use anyhow::{bail, Ok, Result};
use clap::Args;
use nix::unistd;

use crate::container::{Container, ContainerStatus};

#[derive(Debug, Args)]
pub struct Start {
    pub container_id: String,
}

impl Start {
    pub fn exec(&self, root_path: PathBuf) -> Result<()> {
        Ok(())
    }
}
