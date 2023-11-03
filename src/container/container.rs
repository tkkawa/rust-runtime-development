use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use crate::container::{ContainerStatus, State};

#[derive(Debug, Clone)]
pub struct Container {
    // State of the container
    pub state: State,
    // indicated the directory for the root path in the container
    pub root: PathBuf,
}

impl Container {
    pub fn new(
        container_id: &str,
        status: ContainerStatus,
        pid: Option<i32>,
        bundle: &str,
        container_root: &PathBuf,
    ) -> Result<(Self)> {
        let container_root = fs::canonicalize(container_root)?;
        let state = State::new(container_id, status, pid, bundle);
        Ok(Self {
            state,
            root: container_root,
        })
    }

    pub fn id(&self) -> &str {
        self.state.id.as_str()
    }

    pub fn status(&self) -> ContainerStatus {
        self.state.status
    }

    pub fn save(&self) -> Result<()> {
        log::debug!("Save container status: {:?} in {:?}", self, self.root);
        self.state.save(&self.root)
    }

    pub fn set_pid(&self, pid: i32) -> Self {
        Self::new(
            self.state.id.as_str(),
            self.state.status,
            Some(pid),
            self.state.bundle.as_str(),
            &self.root,
        )
        .expect("unexpected error")
    }

    pub fn update_status(&self, status: ContainerStatus) -> Result<Self> {
        Self::new(
            self.state.id.as_str(),
            status,
            self.state.pid,
            self.state.bundle.as_str(),
            &self.root,
        )
    }

    pub fn can_start(&self) -> bool {
        self.state.status.can_start()
    }

    pub fn load(container_root: PathBuf) -> Result<Self> {
        let state = State::load(&container_root)?;
        Ok(Self {
            state,
            root: container_root,
        })
    }
}
