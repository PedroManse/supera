use crate::queue::{QueueEventLoopError, QueueRunner};
use crate::{CmdRst, Command, CommandRunner};
use std::any::Any;
use std::fmt;
use std::sync::mpsc::{self, Receiver, RecvError, SendError, Sender};
use std::thread::JoinHandle;

type SR<Cmd> = mpsc::Receiver<Cmd>;
type SS<Cmd> = mpsc::Sender<CmdRst<Cmd>>;

/// API of [`QueueRunner`] for managing a single runner
pub struct SingleQueueAPI<Cmd>
where
    Cmd: Command,
{
    send_cmd: Sender<Cmd>,
    recv_res: Receiver<CmdRst<Cmd>>,
    thread: JoinHandle<Result<QueueRunner<Cmd, SR<Cmd>, SS<Cmd>>, QueueEventLoopError>>,
}

#[derive(Debug)]
pub enum SingleQueueCloseError<Cmd>
where
    Cmd: Command,
{
    Send(SendError<Cmd>),
    Join(Box<dyn Any + Send>),
    Worker(QueueEventLoopError),
}

impl<Cmd: Command> fmt::Display for SingleQueueCloseError<Cmd> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Join(e) => write!(f, "Failed to join thread, {e:?}"),
            Self::Send(cmd) => write!(f, "Failed to send command {cmd}"),
            Self::Worker(e) => write!(f, "Worker failed with: {e}"),
        }
    }
}

impl<Cmd: fmt::Debug + Command> std::error::Error for SingleQueueCloseError<Cmd> {}

impl<Cmd> CommandRunner for SingleQueueAPI<Cmd>
where
    Cmd: Command,
{
    type Cmd = Cmd;
    type SendAck = Result<(), SendError<Cmd>>;
    type CloseResult = Result<QueueRunner<Cmd, SR<Cmd>, SS<Cmd>>, SingleQueueCloseError<Cmd>>;
    unsafe fn new() -> Self {
        let (send_cmd, recv_cmd) = mpsc::channel();
        let (send_res, recv_res) = mpsc::channel();
        let thread = QueueRunner::spawn(recv_cmd, send_res);
        SingleQueueAPI {
            send_cmd,
            recv_res,
            thread,
        }
    }
    fn send(&self, cmd: Self::Cmd) -> Self::SendAck {
        self.send_cmd.send(cmd)
    }
    fn close_with(self, mut s: impl crate::StopRunner<Self::Cmd>) -> Self::CloseResult {
        let cmd = s.get();
        self.send_cmd
            .send(cmd)
            .map_err(SingleQueueCloseError::Send)?;
        self.thread
            .join()
            .map_err(SingleQueueCloseError::Join)?
            .map_err(SingleQueueCloseError::Worker)
    }
}

impl<Cmd> SingleQueueAPI<Cmd>
where
    Cmd: Command,
{
    /// # Errors
    /// An error would occour if the [runner](QueueRunner) was closed but the [api](QueueAPI) was not dropped.
    pub fn recv(&self) -> Result<CmdRst<Cmd>, RecvError> {
        self.recv_res.recv()
    }
    /// # Errors
    /// An error would occour if the [runner](QueueRunner) was closed but the [api](QueueAPI) was not dropped.
    pub fn try_recv(&self) -> Result<CmdRst<Cmd>, mpsc::TryRecvError> {
        self.recv_res.try_recv()
    }
}
