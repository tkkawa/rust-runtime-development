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

pub async fn prepare_rootfs(spec: &Spec, rootfs: &PathBuf, bind_devices: bool) -> Result<()> {
    let mut flags = MsFlags::MS_REC;
    match spec.linux {
        Some(ref linux) => match linux.rootfs_propagation.as_ref() {
            "shared" => flags |= MsFlags::MS_SHARED,
            "private" => flags |= MsFlags::MS_PRIVATE,
            "slave" | "" => flags |= MsFlags::MS_SLAVE,
            _ => panic!(),
        },
        None => flags |= MsFlags::MS_SLAVE,
    };
    let linux = spec.linux.as_ref().unwrap();
    mount(None::<&str>, "/", None::<&str>, flags, None::<&str>)?;

    // log::debug!("mount root fs {:?}", rootfs);
    mount(
        Some(rootfs),
        rootfs,
        None::<&str>,
        MsFlags::MS_BIND | MsFlags::MS_REC,
        None::<&str>,
    )?;

    for m in &spec.mounts {
        let (flags, data) = parse_mount(m);
        // log::debug!("mount {:?}", m);
        if m.typ == "cgroup" {
            log::warn!("A feature of cgoup is unimplemented.");
            // skip
        } else if m.destination == PathBuf::from("/dev") {
            mount_from(
                m,
                rootfs,
                flags & !MsFlags::MS_RDONLY,
                &data,
                &linux.mount_label,
            )?;
        } else {
            mount_from(m, rootfs, flags, &data, &linux.mount_label)?;
        }
    }

    let olddir = getcwd()?;
    chdir(rootfs)?;

    setup_default_symlinks(rootfs)?;
    create_devices(&linux.devices, bind_devices).await?;
    setup_ptmx(rootfs)?;

    chdir(&olddir)?;

    Ok(())
}

pub fn pivot_rootfs<P: ?Sized + NixPath>(path: &P) -> Result<()> {
    let newroot = open(path, OFlag::O_DIRECTORY | OFlag::O_RDONLY, Mode::empty())?;

    pivot_root(path, path)?;

    umount2("/", MntFlags::MNT_DETACH)?;
    fchdir(newroot)?;
    Ok(())
}

