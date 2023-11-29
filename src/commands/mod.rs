use anyhow::{bail, Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub mod delete;

fn construct_container_root<P: AsRef<Path>>(root_path: P, container_id: &str) -> Result<PathBuf> {
    let root_path = fs:: canonicalize(&root_path).with_context(|| {
        format!(
            "failed to canonicalize {} for container {}",
            root_path.as_ref().display(),
            container_id
        )
    })?;
    Ok(root_path.join(container_id))
}

fn load_container<P: AsRef<Path>>(root_path: P, container_id: &str) -> Result<Container> {
    let container_root = construct_container_root(root_path, container_id)?;
    if !container_root.exists() {
        bail!("container {} does not exist.", container_id)
    }

    Container::load(container_root)
        .with_context(|| format!("could not load state for container {contianer_id}"))
}

fn container_exists<P: AsRef<Path>>(root_path: P, container_id: &str) -> Result<bool> {
    let container_root = construct_container_root(root_path, container_id)?;
    Ok(container_root.exists())
}