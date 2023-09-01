use std::os::unix::io::RawFd;

use anyhow::Result;
use nix::fcntl::OFlag;
use nix::unistd::pipe2;

pub struct Cond {
    rfd: RawFd,
    wfd: RawFd,
}

impl Cond {
    pub fn new() -> Result<Cond> {
        let (rfd, wfd) = pipe2(OFlag::O_CLOEXEC)?;
        Ok(Cond { rfd, wfd })
    }
}
