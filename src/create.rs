use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use anyhow::{bail, Ok, Result};
use clap::Args;
use nix::fcntl;
use nix::sched;
use nix::sys::stat;
use nix::unistd;
use nix::unistd::{sethostname, Gid, Uid};

use crate::container::{Container, ContainerStatus};
use crate::notify_socket::NotifyListener;
use crate::process::Process;
use crate::process::{fork::fork_first, fork::fork_init};
use crate::rootfs;
use crate::spec;
use crate::stdio::FileDescriptor;
use crate::tty;
use crate::utils;

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
) -> Result<Process> {
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
    )? {
        Process::Parent(parent) => Ok(Process::Parent(parent)),
        Process::Child(child) => {
            setid(Uid::from_raw(0), Gid::from_raw(0))?;
            if let Some(csocketfd) = csocketfd {
                tty::ready(csocketfd)?;
            }
            log::debug!("Ready TTY");

            for &(space, fd) in &to_enter {
                sched::setns(fd, space)?;
                log::debug!("setns()->space:{:?},fd:{:?}", space, fd);
                unistd::close(fd)?;
                if space == sched::CloneFlags::CLONE_NEWUSER {
                    setid(Uid::from_raw(0), Gid::from_raw(0))?;
                    log::debug!("Set ID");
                }
            }

            sched::unshare(cf & !sched::CloneFlags::CLONE_NEWUSER)?;
            log::debug!("Unshare!");

            match fork_init(child)? {
                Process::Child(child) => Ok(Process::Child(child)),
                Process::Init(mut init) => {
                    log::debug!("Execute Rootfs");
                    futures::executor::block_on(rootfs::prepare_rootfs(
                        spec,
                        rootfs,
                        cf.contains(sched::CloneFlags::CLONE_NEWUSER),
                    ))?;
                    rootfs::pivot_rootfs(&*rootfs)?;
                    log::debug!("Complete Pivot Root");

                    init.ready()?;
                    log::debug!("Init Ready");

                    notify_socket.wait_for_container_start()?;
                    // if spec.process.no_new_privileges {
                    //     let _ = prctl::set_no_new_privileges(true);
                    // }

                    // // set hostname and environment value
                    // sethostname(&spec.hostname)?;
                    // utils::set_env_val(&spec.process.env);

                    // setid(
                    //     Uid::from_raw(spec.process.user.uid),
                    //     Gid::from_raw(spec.process.user.gid),
                    // )?;
                    // capabilities::reset_effective()?;
                    // if let Some(caps) = &spec.process.capabilities {
                    //     capabilities::set_capabilities(&caps)?;
                    // }
                    // // set rlimits
                    // for rlimit in &spec.process.rlimits {
                    //     utils::set_rlimits(rlimit)?;
                    // }

                    utils::do_exec(&spec.process.args[0], &spec.process.args)?;
                    container.update_status(ContainerStatus::Stopped)?.save()?;
                    log::debug!("update");

                    Ok(Process::Init(init))
                }
                Process::Parent(_) => unreachable!(),
            }
        }
        _ => unreachable!(),
    }
}

fn setid(uid: Uid, gid: Gid) -> Result<()> {
    if let Err(e) = prctl::set_keep_capabilities(true) {
        bail!("set keep capabilities returned {}", e);
    };
    unistd::setresgid(gid, gid, gid)?;
    unistd::setresuid(uid, uid, uid)?;

    if let Err(e) = prctl::set_keep_capabilities(false) {
        bail!("set keep capabilities returned {}", e);
    };
    Ok(())
}
