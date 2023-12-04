use super::{Container, ContainerStatus};
use crate::{error::LibcontainerError, signal::Signal};


impl Container {
    pub (crate) fn do_kill<S: Into<Signal>>(
        &self,
        signal: S,
        all: bool,
    ) -> Result<(), LibcontainerError> {
        
    }
}