use crate::queue::QueueRunner;
use crate::{CmdRst, Command, CommandRunner};
use crossbeam_channel as mpmc;
use std::any::Any;
use std::sync::mpsc;
use std::thread::JoinHandle;

type MR<Cmd> = mpmc::Receiver<Cmd>;
type SS<Cmd> = mpsc::Sender<CmdRst<Cmd>>;
type PoolRunner<Cmd> = QueueRunner<Cmd, MR<Cmd>, SS<Cmd>>;

/// API of [`QueueRunner`] for managing multiple runners
pub struct PoolQueueAPI<Cmd, const N: usize>
where
    Cmd: Command,
{
    send_cmd: mpmc::Sender<Cmd>,
    recv_res: mpsc::Receiver<CmdRst<Cmd>>,
    runners: [JoinHandle<PoolRunner<Cmd>>; N],
}

#[derive(Debug)]
pub enum PoolQueueCloseError<Cmd>
where
    Cmd: Command,
{
    Send(mpmc::SendError<Cmd>),
    Join(Box<dyn Any + Send>),
}

impl<Cmd, const N: usize> CommandRunner for PoolQueueAPI<Cmd, N>
where
    Cmd: Command,
{
    type Cmd = Cmd;
    type SendAck = Result<(), mpmc::SendError<Cmd>>;
    type CloseResult =
        Result<[Result<PoolRunner<Cmd>, Box<dyn Any + Send>>; N], mpmc::SendError<Cmd>>;
    unsafe fn new() -> Self {
        let (tx_cmd, rx_cmd) = mpmc::unbounded();
        let (tx_res, rx_res) = mpsc::channel();
        let runners = [(); N].map(|()| QueueRunner::spawn(rx_cmd.clone(), tx_res.clone()));
        Self {
            send_cmd: tx_cmd,
            recv_res: rx_res,
            runners,
        }
    }
    fn send(&self, cmd: Self::Cmd) -> Self::SendAck {
        self.send_cmd.send(cmd)
    }
    fn close_with(self, mut s: impl crate::StopRunner<Self::Cmd>) -> Self::CloseResult {
        for _ in 0..self.runners.len() {
            self.send(s.get())?;
        }
        Ok(self.runners.map(std::thread::JoinHandle::join))
    }
}

impl<Cmd, const N: usize> PoolQueueAPI<Cmd, N>
where
    Cmd: Command,
{
    /// # Errors
    /// An error would occour if the [runner](QueueRunner) was closed but the [api](QueueAPI) was not dropped.
    pub fn recv(&self) -> Result<CmdRst<Cmd>, mpsc::RecvError> {
        self.recv_res.recv()
    }
    /// # Errors
    /// An error would occour if the [runner](QueueRunner) was closed but the [api](QueueAPI) was not dropped.
    pub fn try_recv(&self) -> Result<CmdRst<Cmd>, mpsc::TryRecvError> {
        self.recv_res.try_recv()
    }
}
