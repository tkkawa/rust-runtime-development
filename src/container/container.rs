use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use anyhow::{Error, Result};
use nix::unistd::Pid;
use procfs::process::{ProcState, Process};

use crate::container::{ContainerStatus, State};
use crate::error::LibcontainerError;

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
        // self.state.id.as_str()
        &self.state.id
    }

    pub fn status(&self) -> ContainerStatus {
        self.state.status
    }

    pub fn set_status(&mut self, status: ContainerStatus) -> &mut Self {
        let created = match (status, self.state.created) {
            (ContainerStatus::Created, None) => Some(Utc::now()),
            _ => self.state.created,
        };

        self.state.created = created;
        self.state.status = status;

        self
    }

    pub fn refresh_status(&mut self) -> Result<(), LibcontainerError> {
        let new_status = match self.pid() {
            Some(pid) => {
                if let Ok(proc) = Process::new(pid.as_raw()) {
                    use procfs::process::ProcState;

                    match proc.stat.state()? {
                        ProcState::Zombie | ProcState::Dead => ContainerStatus::Stopped,
                        _ => match self.status() {
                            ContainerStatus::Creating | ContainerStatus::Created | ContainerStatus::Paused => self.status(),
                            _ => ContainerStatus::Running,
                        },
                    }
                } else {
                    ContainerStatus::Stopped
                }
            }
            None => ContainerStatus::Stopped,
        };
        self.set_status(new_status);
        Ok(())

    }

    pub fn save(&self) -> Result<(), LibcontainerError> {
        log::debug!("Save container status: {:?} in {:?}", self, self.root);
        self.state.save(&self.root);
        Ok(())
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

    pub fn pid(&self) -> Option<Pid> {
        self.state.pid.map(Pid::from_raw)
    }

    pub fn load(container_root: PathBuf) -> Result<Self, LibcontainerError> {
        let state = State::load(&container_root)?;
        let mut container = Self {
            state,
            root: container_root,
        };
        container.refresh_status()?;
        Ok(container)
    }
}
