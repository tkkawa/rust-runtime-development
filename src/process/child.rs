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

    pub fn setup_uds(&mut self) -> Result<Sender> {
        let (sender, mut receiver) = pipe::new()?;
        let poll = Poll::new()?;
        poll.registry()
            .register(&mut receiver, CHILD, Interest::READABLE)?;
        self.receiver = Some(receiver);
        self.poll = Some(poll);
        Ok(sender)
    }
}
