use anyhow::{Ok, Result};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;

pub const NOTIFY_FILE: &str = "notify.sock";

pub struct NotifyListener {
    socket: UnixListener,
}

impl NotifyListener {
    pub fn new(root: &PathBuf) -> Result<(Self)> {
        // /var/run/youki/${container_id}/notify.sock
        let _notify_file_path = root.join(NOTIFY_FILE);
        let stream = UnixListener::bind(NOTIFY_FILE)?;
        Ok(Self { socket: stream })
    }
}
