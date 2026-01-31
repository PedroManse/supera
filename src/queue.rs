use crate::{ActionResult, ChanRecv, ChanSend, CmdRst, Command};
use std::fmt;
use std::marker::PhantomData;
use std::thread::JoinHandle;

/// Runner that sends responses to a queue
pub struct QueueRunner<Cmd, R, S>
where
    Cmd: Command,
    R: ChanRecv<Cmd>,
    S: ChanSend<CmdRst<Cmd>>,
{
    pub(crate) d: PhantomData<Cmd>,
    pub(crate) recv_cmd: R,
    pub(crate) send_res: S,
}

#[derive(Debug)]
pub enum QueueEventLoopError {
    SendErr,
    RecvErr,
    ThreadPanic(Box<dyn std::any::Any + Send>),
}

impl fmt::Display for QueueEventLoopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RecvErr => write!(f, "Failed to recieve"),
            Self::SendErr => write!(f, "Failed to write"),
            Self::ThreadPanic(..) => write!(f, "Worker panicked"),
        }
    }
}

impl std::error::Error for QueueEventLoopError {}

impl<Cmd, S, R> QueueRunner<Cmd, R, S>
where
    Cmd: Command,
    R: ChanRecv<Cmd>,
    S: ChanSend<CmdRst<Cmd>>,
{
    pub(crate) fn get(&self) -> Result<Cmd, R::Err> {
        self.recv_cmd.recv_t()
    }
    pub(crate) fn send(&self, res: CmdRst<Cmd>) -> Result<(), S::Err> {
        self.send_res.send_t(res)
    }
    pub(crate) fn exec(cmd: Cmd) -> ActionResult<Cmd> {
        cmd.execute()
    }
}

impl<Cmd, R, S> QueueRunner<Cmd, R, S>
where
    Cmd: Command,
    R: ChanRecv<Cmd> + Send + 'static,
    S: ChanSend<CmdRst<Cmd>> + Send + 'static,
    <R as ChanRecv<Cmd>>::Err: std::fmt::Debug,
    <S as ChanSend<Cmd::Result>>::Err: std::fmt::Debug,
{
    /// # Panics
    /// The default runners panic if the channels they're bound to are dropped.
    pub(crate) fn spawn(recv_cmd: R, send_res: S) -> JoinHandle<Result<Self, QueueEventLoopError>> {
        std::thread::spawn(|| {
            let runner = Self {
                recv_cmd,
                send_res,
                d: PhantomData,
            };
            loop {
                let cmd = runner.get().map_err(|_| QueueEventLoopError::RecvErr)?;
                let r = Self::exec(cmd);
                let ActionResult::Normal(res) = r else { break };
                runner.send(res).map_err(|_| QueueEventLoopError::SendErr)?;
            }
            Ok(runner)
        })
    }
}
