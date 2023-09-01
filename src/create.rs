use std::path::{Path, PathBuf};
use std::fs;

use anyhow::{bail, Ok, Result};
use clap::Args;
use nix::fcntl;
use nix::sys::stat;
use nix::sched;
use nix::unistd;

use crate::container::{Container, ContainerStatus};
use crate::notify_socket::NotifyListener;
use crate::process::fork::fork_first;
use crate::spec;
use crate::tty;
use crate::stdio::FileDescriptor;

#[derive(Debug, Args)]
pub struct Create {
    #[clap(short, long)]
    pid_file: Option<String>,
    #[clap(short, long, default_value = ".")]
    bundle: PathBuf,
    #[clap(short, long)]
    console_socket: Option<String>,
    pub container_id: String,
}

impl Create {
    pub fn exec(&self, root_path: PathBuf) -> Result<()> {
        let container_dir = root_path.join(&self.container_id);
        if !container_dir.exists() {
            fs::create_dir(&container_dir).unwrap();
        } else {
            bail!("{} already exists", self.container_id)
        }
        unistd::chdir(&self.bundle)?;
        let spec = spec::Spec::load("config.json")?;

        let container_dir = fs::canonicalize(container_dir)?;
        unistd::chdir(&*container_dir)?;
        log::debug!("Create: {:?}", container_dir);

        let container = Container::new(
            &self.container_id,
            ContainerStatus::Creating,
            None,
            self.bundle.to_str().unwrap(),
            &container_dir,
        )?;
        container.save()?;

        let mut notify_socket: NotifyListener = NotifyListener::new(&container_dir)?;

        let rootfs = fs::canonicalize(&spec.root.path)?;
        log::debug!("Create rootfs: {:?}", spec.root.path);

        let (csocketfd, _consolefd) = {
            if let Some(console_socket) = &self.console_socket {
                let (csocketfd, consolefd) =
                    tty::load_console_sockets(&container_dir, console_socket)?;
                (Some(csocketfd), Some(consolefd))
            } else {
                (None, None)
            }
        };

        let process = run_container(
            self.pid_file.as_ref(),
            &mut notify_socket,
            &rootfs,
            &spec,
            csocketfd,
            container,
        )?;

        Ok(())
    }
}

fn run_container<P: AsRef<Path>>(
    pid_file: Option<P>,
    notify_socket: &mut NotifyListener,
    rootfs: &PathBuf,
    spec: &spec::Spec,
    csocketfd: Option<FileDescriptor>,
    container: Container,
) -> Result<()> {
    prctl::set_dumpable(false).unwrap();
    let linux = spec.linux.as_ref().unwrap();
    let mut cf = sched::CloneFlags::empty();
    let mut to_enter = Vec::new();

    for ns in &linux.namespaces {
        let space = sched::CloneFlags::from_bits_truncate(ns.typ as i32);
        if ns.path.is_empty() {
            cf |= space;
        } else {
            let fd = fcntl::open(&*ns.path, fcntl::OFlag::empty(), stat::Mode::empty()).unwrap();
            to_enter.push((space, fd));
            log::debug!("to_enter->space:{:?},fd:{:?}", space, fd);
        }
    }

    match fork_first(
        pid_file,
        cf.contains(sched::CloneFlags::CLONE_NEWUSER),
        linux,
        &container,
    ) {
        _ => unreachable!(),
    }

    Ok(())
}
