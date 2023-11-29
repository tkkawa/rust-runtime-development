use anyhow::{bail, Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub mod delete;

fn container_exists<P: AsRef<Path>>(root_path: P, container_id: &str) -> Result<bool> {
    Ok(())
}