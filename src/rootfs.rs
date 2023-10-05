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

fn default_devices() -> Vec<LinuxDevice> {
    vec![
        LinuxDevice {
            path: "/dev/null".to_string(),
            typ: LinuxDeviceType::C,
            major: 1,
            minor: 3,
            file_mode: Some(0o066),
            uid: None,
            gid: None,
        },
        LinuxDevice {
            path: "/dev/zero".to_string(),
            typ: LinuxDeviceType::C,
            major: 1,
            minor: 5,
            file_mode: Some(0o066),
            uid: None,
            gid: None,
        },
        LinuxDevice {
            path: "/dev/full".to_string(),
            typ: LinuxDeviceType::C,
            major: 1,
            minor: 7,
            file_mode: Some(0o066),
            uid: None,
            gid: None,
        },
        LinuxDevice {
            path: "/dev/tty".to_string(),
            typ: LinuxDeviceType::C,
            major: 5,
            minor: 0,
            file_mode: Some(0o066),
            uid: None,
            gid: None,
        },
        LinuxDevice {
            path: "/dev/urandom".to_string(),
            typ: LinuxDeviceType::C,
            major: 1,
            minor: 9,
            file_mode: Some(0o066),
            uid: None,
            gid: None,
        },
        LinuxDevice {
            path: "/dev/random".to_string(),
            typ: LinuxDeviceType::C,
            major: 1,
            minor: 8,
            file_mode: Some(0o066),
            uid: None,
            gid: None,
        },
    ]
}

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

fn setup_ptmx(rootfs: &PathBuf) -> Result<()> {
    if let Err(e) = remove_file(rootfs.join("dev/ptmx")) {
        if e.kind() != ::std::io::ErrorKind::NotFound {
            let msg = "could not delete /dev/ptmx".to_string();
            panic!("{}", msg);
        }
    }
    symlink("pts/ptmx", rootfs.join("dev/ptmx"))?;
    Ok(())
}

fn setup_default_symlinks(rootfs: &PathBuf) -> Result<()> {
    if Path::new("/proc/kcore").exists() {
        symlink("/proc/kcore", "dev/kcore")?;
    }

    let defaults = [
        ("/proc/self/fd", "dev/fd"),
        ("/proc/self/fd/0", "dev/stdin"),
        ("/proc/self/fd/1", "dev/stdout"),
        ("/proc/self/fd/2", "dev/stderr"),
    ];
    for &(src, dst) in defaults.iter() {
        symlink(src, rootfs.join(dst))?;
    }
    Ok(())
}

async fn create_devices(devices: &[LinuxDevice], bind: bool) -> Result<()> {
    let old_mode = umask(Mode::from_bits_truncate(0o000));
    if bind {
        future::try_join_all(default_devices().iter().chain(devices).map(|dev| {
            if !dev.path.starts_with("/dev") || dev.path.contains("..") {
                panic!("{} is not a valid device path", dev.path);
            }
            bind_dev(dev)
        }))
        .await?;
    } else {
        future::try_join_all(default_devices().iter().chain(devices).map(|dev| {
            if !dev.path.starts_with("/dev") || dev.path.contains("..") {
                panic!("{} is not a valid device path", dev.path);
            }
            mknod_dev(dev)
        }))
        .await?;
    }
    umask(old_mode);
    Ok(())
}

async fn bind_dev(dev: &LinuxDevice) -> Result<()> {
    let fd = open(
        &dev.path[1..],
        OFlag::O_RDWR | OFlag::O_CREAT,
        Mode::from_bits_truncate(0o644),
    )?;
    close(fd)?;
    mount(
        Some(&*dev.path),
        &dev.path[1..],
        None::<&str>,
        MsFlags::MS_BIND,
        None::<&str>,
    )?;
    Ok(())
}

async fn mknod_dev(dev: &LinuxDevice) -> Result<()> {
    fn makedev(major: u64, minor: u64) -> u64 {
        (minor & 0xff) | ((major & 0xfff) << 8) | ((minor & !0xff) << 12) | ((major & !0xfff) << 32)
    }

    let f = to_sflag(dev.typ)?;
    mknod(
        &dev.path[1..],
        f,
        Mode::from_bits_truncate(dev.file_mode.unwrap_or(0)),
        makedev(dev.major, dev.minor),
    )?;
    chown(
        &dev.path[1..],
        dev.uid.map(Uid::from_raw),
        dev.gid.map(Gid::from_raw),
    )?;
    Ok(())
}

fn mount_from(m: &Mount, rootfs: &PathBuf, flags: MsFlags, data: &str, label: &str) -> Result<()> {
    let d;
    if !label.is_empty() && m.typ != "proc" && m.typ != "sysfs" {
        if data.is_empty() {
            d = format! {"context=\"{}\"", label};
        } else {
            d = format! {"{},context=\"{}\"", data, label};
        }
    } else {
        d = data.to_string();
    }

    let dest_for_host = format!(
        "{}{}",
        rootfs.to_string_lossy().into_owned(),
        m.destination.display()
    );
    // log::debug!("dest_for_host: {:?}", dest_for_host);
    let dest = Path::new(&dest_for_host);

    let src = if m.typ == "bind" {
        let src = canonicalize(&m.source)?;
        let dir = if src.is_file() {
            Path::new(&dest).parent().unwrap()
        } else {
            Path::new(&dest)
        };
        create_dir_all(&dir).unwrap();
        // make sure file exists so we can bind over it
        if src.is_file() {
            OpenOptions::new()
                .create(true)
                .write(true)
                .open(&dest)
                .unwrap();
        }
        src
    } else {
        create_dir_all(&dest).unwrap();
        PathBuf::from(&m.source)
    };
    // log::debug!("src: {:?}", src);

    if let Err(::nix::Error::Sys(errno)) = mount(Some(&*src), dest, Some(&*m.typ), flags, Some(&*d))
    {
        if errno != Errno::EINVAL {
            let chain = format!("mount of {} failed", m.destination.display());
            panic!("{}", chain);
        }
        // try again without mount label
        mount(Some(&*src), dest, Some(&*m.typ), flags, Some(data))?;
    }
    // remount bind mounts if they have other flags (like MsFlags::MS_RDONLY)
    if flags.contains(MsFlags::MS_BIND)
        && flags.intersects(
            !(MsFlags::MS_REC
                | MsFlags::MS_REMOUNT
                | MsFlags::MS_BIND
                | MsFlags::MS_PRIVATE
                | MsFlags::MS_SHARED
                | MsFlags::MS_SLAVE),
        )
    {
        mount(
            Some(&*dest),
            &*dest,
            None::<&str>,
            flags | MsFlags::MS_REMOUNT,
            None::<&str>,
        )?;
    }
    Ok(())
}

pub fn pivot_rootfs<P: ?Sized + NixPath>(path: &P) -> Result<()> {
    let newroot = open(path, OFlag::O_DIRECTORY | OFlag::O_RDONLY, Mode::empty())?;

    pivot_root(path, path)?;

    umount2("/", MntFlags::MNT_DETACH)?;
    fchdir(newroot)?;
    Ok(())
}

fn to_sflag(t: LinuxDeviceType) -> Result<SFlag> {
    Ok(match t {
        LinuxDeviceType::B => SFlag::S_IFBLK,
        LinuxDeviceType::C | LinuxDeviceType::U => SFlag::S_IFCHR,
        LinuxDeviceType::P => SFlag::S_IFIFO,
        LinuxDeviceType::A => bail!("type a is not allowed for linux device"),
    })
}

fn parse_mount(m: &Mount) -> (MsFlags, String) {
    let mut flags = MsFlags::empty();
    let mut data = Vec::new();
    for s in &m.options {
        if let Some((is_clear, flag)) = match s.as_str() {
            "defaults" => Some((false, MsFlags::empty())),
            "ro" => Some((false, MsFlags::MS_RDONLY)),
            "rw" => Some((true, MsFlags::MS_RDONLY)),
            "suid" => Some((true, MsFlags::MS_NOSUID)),
            "nosuid" => Some((false, MsFlags::MS_NOSUID)),
            "dev" => Some((true, MsFlags::MS_NODEV)),
            "nodev" => Some((false, MsFlags::MS_NODEV)),
            "exec" => Some((true, MsFlags::MS_NOEXEC)),
            "noexec" => Some((false, MsFlags::MS_NOEXEC)),
            "sync" => Some((false, MsFlags::MS_SYNCHRONOUS)),
            "async" => Some((true, MsFlags::MS_SYNCHRONOUS)),
            "dirsync" => Some((false, MsFlags::MS_DIRSYNC)),
            "remount" => Some((false, MsFlags::MS_REMOUNT)),
            "mand" => Some((false, MsFlags::MS_MANDLOCK)),
            "nomand" => Some((true, MsFlags::MS_MANDLOCK)),
            "atime" => Some((true, MsFlags::MS_NOATIME)),
            "noatime" => Some((false, MsFlags::MS_NOATIME)),
            "diratime" => Some((true, MsFlags::MS_NODIRATIME)),
            "nodiratime" => Some((false, MsFlags::MS_NODIRATIME)),
            "bind" => Some((false, MsFlags::MS_BIND)),
            "rbind" => Some((false, MsFlags::MS_BIND | MsFlags::MS_REC)),
            "unbindable" => Some((false, MsFlags::MS_UNBINDABLE)),
            "runbindable" => Some((false, MsFlags::MS_UNBINDABLE | MsFlags::MS_REC)),
            "private" => Some((false, MsFlags::MS_PRIVATE)),
            "rprivate" => Some((false, MsFlags::MS_PRIVATE | MsFlags::MS_REC)),
            "shared" => Some((false, MsFlags::MS_SHARED)),
            "rshared" => Some((false, MsFlags::MS_SHARED | MsFlags::MS_REC)),
            "slave" => Some((false, MsFlags::MS_SLAVE)),
            "rslave" => Some((false, MsFlags::MS_SLAVE | MsFlags::MS_REC)),
            "relatime" => Some((false, MsFlags::MS_RELATIME)),
            "norelatime" => Some((true, MsFlags::MS_RELATIME)),
            "strictatime" => Some((false, MsFlags::MS_STRICTATIME)),
            "nostrictatime" => Some((true, MsFlags::MS_STRICTATIME)),
            _ => None,
        } {
            if is_clear {
                flags &= !flag;
            } else {
                flags |= flag;
            }
        } else {
            data.push(s.as_str());
        };
    }
    (flags, data.join(","))
}
