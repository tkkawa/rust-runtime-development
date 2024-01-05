use super::{Container, ContainerStatus};
use crate::{error::LibcontainerError, signal::Signal};
use nix::sys::signal;

impl Container {
    pub (crate) fn do_kill<S: Into<Signal>>(
        &self,
        signal: S,
        all: bool,
    ) -> Result<(), LibcontainerError> {
        self.kill_one_process(signal)
    }

    fn kill_one_process<S: Into<Signal>>(&self, signal: S) -> Result<(), LibcontainerError> {
        let signal = signal.into().into_raw();
        let pid = self.pid().unwrap();

        tracing::debug!("kill signal {} to {}", signal, pid);
        match signal::kill(pid, signal) {
            Ok(_) => {}
            // Err(nix::errno::Errno::ESRCH) => {
            //     // the process does not exist, which is what we want
            // }
            Err(err) => {
                tracing::error!(id = ?self.id(), err = ?err, ?pid, ?signal, "failed to kill process");
                return Err(LibcontainerError::OtherSyscall(err));
            } 
        }
        Ok(())
    }
}
