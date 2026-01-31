use std::fmt;
use std::marker::PhantomData;
use std::thread::JoinHandle;

use crate::{ActionResult, ChanRecv, CmdRst, Command};

pub(crate) type InternalCommandLink<Cmd> = oneshot::Sender<CmdRst<Cmd>>;
pub(crate) type ExternalCommandLink<Cmd> = oneshot::Receiver<CmdRst<Cmd>>;

pub struct QueuedCommand<Cmd>
where
    Cmd: Command,
{
    pub(crate) cmd: Cmd,
    pub(crate) chan: InternalCommandLink<Cmd>,
}

#[derive(Debug)]
pub enum OneshotEventLoopError<Cmd>
where
    Cmd: Command,
{
    SendErr(CmdRst<Cmd>),
    RecvErr,
    ThreadPanic(Box<dyn std::any::Any + Send>),
}

impl<Cmd: Command + fmt::Debug> fmt::Display for OneshotEventLoopError<Cmd> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RecvErr => write!(f, "Failed to recieve"),
            Self::SendErr(t) => write!(f, "Failed to write ({t:?})"),
            Self::ThreadPanic(..) => write!(f, "Worker panicked"),
        }
    }
}

impl<Cmd: Command + fmt::Debug> std::error::Error for OneshotEventLoopError<Cmd> {}

impl<Cmd> std::fmt::Debug for QueuedCommand<Cmd>
where
    Cmd: std::fmt::Debug + Command,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Queued Command ({:?})", self.cmd)
    }
}

#[derive(Debug)]
pub struct OneShotRunner<Cmd, R>
where
    Cmd: Command,
    R: ChanRecv<QueuedCommand<Cmd>>,
{
    d: PhantomData<Cmd>,
    pub(crate) reqs: R,
}

impl<Cmd, R> OneShotRunner<Cmd, R>
where
    Cmd: Command,
    R: ChanRecv<QueuedCommand<Cmd>> + Send + 'static,
    <R as ChanRecv<QueuedCommand<Cmd>>>::Err: std::fmt::Debug,
{
    fn get(&self) -> Result<QueuedCommand<Cmd>, R::Err> {
        self.reqs.recv_t()
    }
    fn exec(cmd: Cmd) -> ActionResult<Cmd> {
        cmd.execute()
    }
    /// # Panics
    /// The default runners panic if the channels they're bound to are dropped.
    pub(crate) fn spawn(rx: R) -> JoinHandle<Result<Self, OneshotEventLoopError<Cmd>>> {
        std::thread::spawn(move || {
            let runner = Self {
                reqs: rx,
                d: PhantomData,
            };
            loop {
                let msg = runner.get().map_err(|_| OneshotEventLoopError::RecvErr)?;
                let r = Self::exec(msg.cmd);
                let ActionResult::Normal(res) = r else { break };
                msg.chan
                    .send(res)
                    .map_err(|k| OneshotEventLoopError::SendErr(k.into_inner()))?;
            }
            Ok(runner)
        })
    }
}
