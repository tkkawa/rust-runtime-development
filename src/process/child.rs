use std::io::Read;
use std::io::Write;
use std::time::Duration;

use anyhow::{bail, Ok, Result};
use mio::unix::pipe;
use mio::unix::pipe::{Receiver, Sender};
use mio::{Events, Interest, Poll, Token};
use nix::unistd::Pid;

use crate::process::message::Message;

const CHILD: Token = Token(1);
pub struct ChildProcess {
    sender_for_parent: Sender,
    receiver: Option<Receiver>,
    poll: Option<Poll>,
}

impl ChildProcess {
    pub fn new(sender_for_parent: Sender) -> Result<Self> {
        Ok(Self {
            sender_for_parent,
            receiver: None,
            poll: None,
        })
    }
}
