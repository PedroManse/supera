use crossbeam_channel as mpmc;
use std::thread::JoinHandle;

use crate::oneshot::{ExternalCommandLink, OneShotRunner, OneshotEventLoopError, QueuedCommand};
use crate::{Command, CommandRunner};
type MR<Cmd> = mpmc::Receiver<QueuedCommand<Cmd>>;

pub struct OneShotPoolAPI<Cmd, const N: usize>
where
    Cmd: Command,
{
    cmd_queue: mpmc::Sender<QueuedCommand<Cmd>>,
    runners: [JoinHandle<Result<OneShotRunner<Cmd, MR<Cmd>>, OneshotEventLoopError<Cmd>>>; N],
}

impl<Cmd, const N: usize> CommandRunner for OneShotPoolAPI<Cmd, N>
where
    Cmd: Command,
{
    type Cmd = Cmd;
    type SendAck = Result<ExternalCommandLink<Cmd>, mpmc::SendError<QueuedCommand<Cmd>>>;
    type CloseResult = Result<
        [Result<OneShotRunner<Cmd, MR<Cmd>>, OneshotEventLoopError<Cmd>>; N],
        mpmc::SendError<QueuedCommand<Cmd>>,
    >;
    unsafe fn new() -> Self {
        let (tx_cmd, rx_cmd) = mpmc::unbounded::<QueuedCommand<Cmd>>();
        let runners = [(); N].map(|()| OneShotRunner::<Cmd, MR<Cmd>>::spawn(rx_cmd.clone()));
        Self {
            cmd_queue: tx_cmd,
            runners,
        }
    }
    fn send(&self, cmd: Self::Cmd) -> Self::SendAck {
        let (tx, rx) = oneshot::channel();
        let msg = QueuedCommand { cmd, chan: tx };
        self.cmd_queue.send(msg)?;
        Ok(rx)
    }
    fn close_with(self, mut s: impl crate::StopRunner<Self::Cmd>) -> Self::CloseResult {
        for _ in 0..self.runners.len() {
            self.send(s.get())?;
        }
        Ok(self
            .runners
            .map(std::thread::JoinHandle::join)
            .map(|e| e.map_err(OneshotEventLoopError::ThreadPanic)?))
    }
}
