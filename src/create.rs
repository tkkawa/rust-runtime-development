use std::path::{Path, PathBuf};
use std::fs;

use anyhow::{bail, Ok, Result};
use clap::Args;
use nix::unistd;

use crate::container::{Container, ContainerStatus};
use crate::notify_socket::NotifyListener;
use crate::spec;

#[derive(Debug, Args)]
pub struct Create {
    #[clap(short, long)]
    pid_file: Option<String>,
    #[clap(short, long, default_value = ".")]
    bundle: PathBuf,
    #[clap(short, long)]
    console_socket: Option<String>,
    pub container_id: String,
}

impl Create {
    pub fn exec(&self, root_path: PathBuf) -> Result<()> {
        let container_dir = root_path.join(&self.container_id);
        if !container_dir.exists() {
            fs::create_dir(&container_dir).unwrap();
        } else {
            bail!("{} already exists", self.container_id)
        }
        unistd::chdir(&self.bundle)?;
        let spec = spec::Spec::load("config.json")?;

        let container_dir = fs::canonicalize(container_dir)?;
        unistd::chdir(&*container_dir)?;
        log::debug!("Create: {:?}", container_dir);

        let container = Container::new(
            &self.container_id,
            ContainerStatus::Creating,
            None,
            self.bundle.to_str().unwrap(),
            &container_dir,
        )?;
        container.save()?;

        let mut notify_socket: NotifyListener = NotifyListener::new(&container_dir)?;

        Ok(())
    }
}
