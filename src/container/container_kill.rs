use super::{Container, ContainerStatus};
use crate::{error::LibcontainerError, signal::Signal};


impl Container {
    pub (crate) fn do_kill<S: Into<Signal>>(
        &self,
        signal: S,
        all: bool,
    ) -> Result<(), LibcontainerError> {
        if all {
            self.kill_all_processes(signal)
        } else {
            self.kill_one_process(signal)
        }
    }

    fn kill_one_process<S: Into<Signal>>(&self, signal: S) -> Result<(), LibcontainerError> {
        
    }

    fn kill_all_processes<S: Into<Signal>>(&self, signal: S) -> Result<(), LibcontainerError> {
        
    }
}