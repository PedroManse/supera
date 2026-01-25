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
    pub(crate) fn spawn(rx: R) -> JoinHandle<Self> {
        std::thread::spawn(move || {
            let runner = Self {
                reqs: rx,
                d: PhantomData,
            };
            loop {
                let msg = runner.get().unwrap();
                let r = Self::exec(msg.cmd);
                let ActionResult::Normal(res) = r else { break };
                msg.chan.send(res).unwrap();
            }
            runner
        })
    }
}
