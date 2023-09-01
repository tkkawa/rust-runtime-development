use std::path::Path;

use crate::container::{Container, ContainerStatus};
use crate::spec;

pub fn fork_first<P: AsRef<Path>>(
    pid_file: Option<P>,
    userns: bool,
    linux: &spec::Linux,
    container: &Container,
) {
    log::debug!("Create Child Process");
}
