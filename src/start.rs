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
        let container_root = root_path.jpin(&self.container_id);
        if ! container_root.exists() {
            bail!("{} doesn't exists.", self.container_id)
        }
        let container = Container::laod(container_root)?.refresh_status()?;
        if !container.can_start() {
            let err_msg = format!(
                "{} counld not be started because it was {:?}",
                container.id(),
                container.status()
            );
            log::error!("{}", err_msg);
            bail!(err_msg);
        }
        Ok(())
    }
}
