use std::path::{Path, PathBuf};

use anyhow::{bail, Ok, Result};
use clap::Args;

#[derive(Debug, Args)]
pub struct Create {

}

impl Create {
    pub fn exec(&self, root_path: PathBuf) -> Result<()> {
        Ok(())
    }
}