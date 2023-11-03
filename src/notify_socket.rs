use std::io::prelude::*;
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixListener;
use std::path::PathBuf;

use anyhow::Result;

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

    pub fn wait_for_container_start(&mut self) -> Result<()> {
        match self.socket.accept() {
            Ok((mut socket, _addr)) => {
                let mut response = String::new();
                socket.read_to_string(&mut response)?;
                log::debug!("receive :{}", response);
            }
            Err(e) => println!("accept function failed: {:?}", e),
        }
        Ok(())
    }
}

pub struct NotifySocket {}

impl NotifySocket {
    pub fn new(_root: PathBuf) -> Result<Self> {
        Ok(Self {})
    }

    pub fn notify_container_start(&mut self) -> Result<()> {
        log::debug!("connection start");
        let mut stream = UnixStream::connect("notify.sock");
        stream.write_all(b"start container")?;
        log::debug!("write finish");
        Ok(())
    }
}
