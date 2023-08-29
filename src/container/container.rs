use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use crate::container::{ContainerStatus, State};

#[derive(Debug, Clone)]
pub struct Container {
    
}

impl Container {
    pub fn new(
        container_id: &str,
        status: ContainerStatus,
        pid: Option<i32>,
        bundle: &str,
        container_root: &PathBuf,
    ) -> Result<()> {
        let container_root = fs::canonicalize(container_root)?;
        let state = State::new(container_id, status, pid, bundle);
        Ok(())
    }
}
