use std::path::PathBuf;

use anyhow::Result;

use crate::container::ContainerStatus;

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
        Ok(())
    }
}
