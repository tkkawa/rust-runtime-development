use std::fs;
use std::io::prelude::*;
use std::path::Path;

use anyhow::bail;
use anyhow::Result;
use libc::exit;
use nix::sched;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd;

use crate::cond::Cond;
use crate::container::{Container, ContainerStatus};
use crate::process::{child, init, parent, Process};
use crate::spec;

pub fn fork_first<P: AsRef<Path>>(
    pid_file: Option<P>,
    userns: bool,
    linux: &spec::Linux,
    container: &Container,
) -> Result<Process> {
    let ccond = Cond::new();

    let (mut parent, sender_for_parent) = parent::ParentProcess::new()?;
    log::debug!("Create Parent Process");

    let child = child::ChildProcess::new(sender_for_parent)?;
    log::debug!("Create Child Process");

    unsafe {
        log::debug!("Call fork systemcall");
        match unistd::fork()? {
            unistd::ForkResult::Child => {
                if let Some(ref r) = linux.resources {
                    if let Some(adj) = r.oom_score_adj {
                        let mut f = fs::File::create("/proc/self/oom_score_adj")?;
                        f.write_all(adj.to_string().as_bytes())?;
                        log::debug!("Create oom_score_adj: {:?}", f);
                    }
                }
                if userns {
                    sched::unshare(sched::CloneFlags::CLONE_NEWUSER)?;
                    log::debug!("Unshare !");
                }
                ccond.notify()?;

                Ok(Process::Child(child))
            }
            unistd::ForkResult::Parent { child } => {
                log::debug!("Wait");
                ccond.wait()?;

                log::debug!("Wait for child ready");
                let init_pid = parent.wait_for_child_ready()?;
                container
                    .update_status(ContainerStatus::Created)?
                    .set_pid(init_pid)
                    .save()?;
                log::debug!(
                    "Save container status: {:?} in {:?}",
                    container,
                    container.root
                );

                if let Some(pid_file) = pid_file {
                    fs::write(&pid_file, format!("{}", child))?;
                    log::debug!("Create pid file: {:?}", child);
                }
                Ok(Process::Parent(parent))
            }
        }
    }
}

pub fn fork_init(mut child_process: child::ChildProcess) -> Result<Process> {
    let sender_for_child = child_process.setup_uds()?;
    unsafe {
        match unistd::fork()? {
            unistd::ForkResult::Child => {
                Ok(Process::Init(init::InitProcess::new(sender_for_child)))
            }
            unistd::ForkResult::Parent { child } => {
                log::debug!("Wait for init ready");
                child_process.wait_for_init_ready()?;
                log::debug!("Ready Child");
                child_process.ready(child)?;
                log::debug!("Wait Pid");
                match waitpid(child, None)? {
                    WaitStatus::Exited(pid, status) => {
                        log::debug!("exited pid: {:?}, status: {:?}", pid, status);
                        exit(status);
                    }
                    WaitStatus::Signaled(pid, status, _) => {
                        log::debug!("signaled pid: {:?}, status: {:?}", pid, status);
                        exit(0);
                    }
                    _ => bail!("abnormal exited!"),
                }
            }
        }
    }
}
