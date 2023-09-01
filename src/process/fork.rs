use std::path::Path;

use crate::cond::Cond;
use crate::container::{Container, ContainerStatus};
use crate::spec;

pub fn fork_first<P: AsRef<Path>>(
    pid_file: Option<P>,
    userns: bool,
    linux: &spec::Linux,
    container: &Container,
) {
    let ccond = Cond::new()?;
    log::debug!("Create Child Process");
}
