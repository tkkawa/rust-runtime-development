use super::{Container, ContainerStatus};
use crate::error::LibcontainerError;
use nix::sys::signal;

impl Container {
    pub fn delete(&mut self, force: bool) -> Result<(), LibcontainerError> {
        self.refresh_status()?;

        tracing::debug!("container status: {:?}", self.status());
        match self.status() {
            ContainerStatus::Stopped => {}
            ContainerStatus::Created => {
                self.do_kill(signal::Signal::SIGKILL, true)?;
            }
        }
    }
}
