use std::fs::OpenOptions;
use std::fs::{canonicalize, create_dir_all, remove_file};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use futures::future;
use nix::errno::Errno;
use nix::fcntl::{open, OFlag};
use nix::mount::MsFlags;
use nix::mount::*;
use nix::sys::stat::{mknod, umask};
use nix::sys::stat::{Mode, SFlag};
use nix::unistd::{chdir, chown, close, fchdir, getcwd, pivot_root};
use nix::unistd::{Gid, Uid};
use nix::NixPath;

use crate::spec::{LinuxDevice, LinuxDeviceType, Mount, Spec};

pub fn pivot_rootfs<P: ?Sized + NixPath>(path: &P) -> Result<()> {
    let newroot = open(path, OFlag::O_DIRECTORY | OFlag::O_RDONLY, Mode::empty())?;

    pivot_root(path, path)?;

    umount2("/", MntFlags::MNT_DETACH)?;
    fchdir(newroot)?;
    Ok(())
}
