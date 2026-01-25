use crossbeam_channel as mpmc;
use std::sync::mpsc;

#[cfg(test)]
mod test;

pub mod oneshot;
pub mod oneshot_pool;
pub mod oneshot_single;

pub(crate) mod queue;
pub mod queue_pool;
pub mod queue_single;

#[derive(Debug)]
pub enum ActionResult<R> {
    Normal(R),
    Stop,
}

pub trait Command: Send + Sync + 'static {
    type Result: Send;
    fn execute(self) -> ActionResult<Self::Result>;
}

/// Creates a command that would halt the command runner>
pub trait StopRunner<C: Command> {
    fn get(&mut self) -> C;
}

pub trait SimpleStop: Command {
    fn make_stop_command() -> Self;
}
struct SimpleCloser;

/// A command runner's API
pub trait CommandRunner {
    /// The command it accepts
    type Cmd: Command;
    /// Result of sending a command to a runner
    type SendAck;
    /// The result of halting a runner
    type CloseResult;

    /// # Safety
    /// Since this *should* start another thread, it's only safe to call this if
    /// [`CommandRunner::close_with`] or [`CommandRunner::close`] are called.
    ///
    /// [`CommandRunner::scope_with`] and [`CommandRunner::scope`] always call `close` on the
    /// runner.
    unsafe fn new() -> Self;
    fn send(&self, cmd: Self::Cmd) -> Self::SendAck;
    fn close_with(self, s: impl StopRunner<Self::Cmd>) -> Self::CloseResult;
    fn close(self) -> Self::CloseResult
    where
        Self::Cmd: SimpleStop,
        Self: Sized,
    {
        self.close_with(SimpleCloser)
    }

    /// No need to remember to .close the runner if you use scope
    fn scope_with(closer: impl StopRunner<Self::Cmd>, f: impl FnOnce(&Self)) -> Self::CloseResult
    where
        Self: Sized,
    {
        let runner = unsafe { Self::new() };
        f(&runner);
        runner.close_with(closer)
    }

    fn scope(f: impl FnOnce(&Self)) -> Self::CloseResult
    where
        Self: Sized,
        Self::Cmd: SimpleStop,
    {
        Self::scope_with(SimpleCloser, f)
    }
}

impl<C> StopRunner<C> for SimpleCloser
where
    C: SimpleStop,
{
    fn get(&mut self) -> C {
        C::make_stop_command()
    }
}

pub(crate) type CmdRst<C> = <C as Command>::Result;

pub trait ChanSend<T> {
    type Err;
    /// # Errors
    /// associated type to account for send errors
    fn send_t(&self, t: T) -> Result<(), Self::Err>;
}

pub trait ChanRecv<T> {
    type Err;
    /// # Errors
    /// associated type to account for recv errors
    fn recv_t(&self) -> Result<T, Self::Err>;
}

impl<T> ChanSend<T> for mpsc::Sender<T> {
    type Err = mpsc::SendError<T>;
    fn send_t(&self, t: T) -> Result<(), Self::Err> {
        self.send(t)
    }
}

impl<T> ChanRecv<T> for mpsc::Receiver<T> {
    type Err = mpsc::RecvError;
    fn recv_t(&self) -> Result<T, Self::Err> {
        self.recv()
    }
}

impl<T> ChanSend<T> for mpmc::Sender<T> {
    type Err = mpmc::SendError<T>;
    fn send_t(&self, t: T) -> Result<(), Self::Err> {
        self.send(t)
    }
}

impl<T> ChanRecv<T> for mpmc::Receiver<T> {
    type Err = mpmc::RecvError;
    fn recv_t(&self) -> Result<T, Self::Err> {
        self.recv()
    }
}
