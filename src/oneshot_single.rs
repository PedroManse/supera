use std::any::Any;
use std::fmt;
use std::sync::mpsc;
use std::thread::JoinHandle;

use crate::oneshot::{ExternalCommandLink, OneShotRunner, QueuedCommand};
use crate::{Command, CommandRunner};
type SR<Cmd> = mpsc::Receiver<QueuedCommand<Cmd>>;

pub struct OneShotAPI<Cmd>
where
    Cmd: Command,
{
    cmd_queue: mpsc::Sender<QueuedCommand<Cmd>>,
    thread: JoinHandle<OneShotRunner<Cmd, SR<Cmd>>>,
}

#[derive(Debug)]
pub enum OneShotCloseError<Cmd>
where
    Cmd: Command,
{
    Send(QueuedCommand<Cmd>),
    Join(Box<dyn Any + Send>),
}

impl<Cmd: Command + fmt::Debug> fmt::Display for OneShotCloseError<Cmd> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Join(e) => write!(f, "Failed to join thread, {e:?}"),
            Self::Send(cmd) => write!(f, "Failed to send command {cmd:?}"),
        }
    }
}

impl<Cmd: fmt::Debug + Command> std::error::Error for OneShotCloseError<Cmd> {}

impl<Cmd> CommandRunner for OneShotAPI<Cmd>
where
    Cmd: Command,
    <Cmd as Command>::Result: fmt::Debug,
{
    type Cmd = Cmd;
    type SendAck = Result<ExternalCommandLink<Cmd>, QueuedCommand<Cmd>>;
    type CloseResult = Result<OneShotRunner<Cmd, SR<Cmd>>, OneShotCloseError<Cmd>>;
    unsafe fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let thread = OneShotRunner::spawn(rx);
        OneShotAPI {
            cmd_queue: tx,
            thread,
        }
    }
    fn send(&self, cmd: Self::Cmd) -> Self::SendAck {
        let (tx, rx) = oneshot::channel();
        let msg = QueuedCommand { cmd, chan: tx };
        self.cmd_queue.send(msg).map_err(|e| e.0)?;
        Ok(rx)
    }
    fn close_with(self, mut c: impl crate::StopRunner<Self::Cmd>) -> Self::CloseResult {
        self.send(c.get()).map_err(OneShotCloseError::Send)?;
        self.thread.join().map_err(OneShotCloseError::Join)
    }
}
