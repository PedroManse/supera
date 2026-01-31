use crate as supera;
use supera::CommandRunner;

#[derive(Debug, Clone, Copy)]
pub enum MathAction {
    Sub(i32, i32),
    Stop,
}

impl supera::SimpleStop for MathAction {
    fn make_stop_command() -> Self {
        MathAction::Stop
    }
}

impl supera::Command for MathAction {
    type Result = i32;
    fn execute(self) -> supera::ActionResult<i32> {
        supera::ActionResult::Normal(match self {
            Self::Sub(a, b) => a - b,
            Self::Stop => return supera::ActionResult::Stop,
        })
    }
}

mod queue {
    use super::*;

    /// # Panics
    /// The runner can panic.
    /// Sending and receiving the messages can panic.
    #[test]
    fn single_values() -> Result<(), Box<dyn std::error::Error>> {
        const COUNT: usize = 500_000;
        let mut outs = Vec::with_capacity(COUNT);
        supera::queue_single::SingleQueueAPI::<MathAction>::scope(|q| {
            let ma = MathAction::Sub(2, 1);
            for _ in 0..COUNT {
                q.send(ma).unwrap();
            }
            for _ in 0..COUNT {
                outs.push(q.recv().unwrap());
            }
            std::thread::yield_now();
        })?;
        assert_eq!(outs, vec![1; COUNT]);
        Ok(())
    }

    /// # Panics
    /// Runner manager can panic on close.
    /// Each runner can panic.
    /// Sending and receiving the messages can panic.
    #[test]
    fn pool_values() -> Result<(), Box<dyn std::error::Error>> {
        const COUNT: usize = 500_000;
        let mut outs = Vec::with_capacity(COUNT);
        let rs = supera::queue_pool::PoolQueueAPI::<MathAction, 2>::scope(|q| {
            let ma = MathAction::Sub(2, 1);
            for _ in 0..COUNT {
                q.send(ma).unwrap();
            }
            for _ in 0..COUNT {
                outs.push(q.recv().unwrap());
            }
        })?;
        assert_eq!(outs, vec![1; COUNT]);
        for r in rs {
            r?;
        }
        Ok(())
    }

    #[test]
    /// # Panics
    /// The runner can panic.
    /// Sending and receiving the messages can panic.
    fn single_manual_close() -> Result<(), Box<dyn std::error::Error>> {
        let rs = unsafe { supera::queue_single::SingleQueueAPI::new() };
        rs.send(MathAction::Sub(3, 2))?;
        rs.recv()?;
        rs.close()?;
        Ok(())
    }

    /// # Panics
    /// Runner manager can panic on close.
    /// Each runner can panic.
    /// Sending and receiving the messages can panic.
    #[test]
    fn pool_manual_close() -> Result<(), Box<dyn std::error::Error>> {
        let rs = unsafe { supera::queue_pool::PoolQueueAPI::<MathAction, 3>::new() };
        rs.send(MathAction::Sub(3, 2))?;
        rs.recv()?;
        for r in rs.close()? {
            r?;
        }
        Ok(())
    }
}

mod oneshot {
    use super::*;
    /// # Panics
    /// The runner can panic.
    /// Sending and receiving the messages can panic.
    #[test]
    fn single_values() -> Result<(), Box<dyn std::error::Error>> {
        const COUNT: usize = 5_000;
        supera::oneshot_single::OneShotAPI::scope(|q| {
            for _ in 0..COUNT {
                let ma = MathAction::Sub(2, 1);
                let mr = q.send(ma).unwrap();
                let r = mr.recv().unwrap();
                assert_eq!(r, 1);
            }
        })?;
        Ok(())
    }

    /// # Panics
    /// Runner manager can panic on close.
    /// Each runner can panic.
    /// Sending and receiving the messages can panic.
    #[test]
    fn pool_values() -> Result<(), Box<dyn std::error::Error>> {
        const COUNT: usize = 50_000;
        let runners = supera::oneshot_pool::OneShotPoolAPI::<MathAction, 10>::scope(|q| {
            for _ in 0..COUNT {
                let ma = MathAction::Sub(2, 1);
                let mr = q.send(ma).unwrap();
                let r = mr.recv().unwrap();
                assert_eq!(r, 1);
            }
        })?;
        for r in runners {
            r?;
        }
        Ok(())
    }

    /// # Panics
    /// The runner can panic.
    /// Sending and receiving the messages can panic.
    #[test]
    fn single_manual_close() -> Result<(), Box<dyn std::error::Error>> {
        use supera::oneshot_single::OneShotAPI;
        const COUNT: usize = 2_500;
        let q = unsafe { OneShotAPI::new() };
        for _ in 0..COUNT {
            let ma = MathAction::Sub(2, 1);
            let mr = q.send(ma).unwrap();
            let r = mr.recv().unwrap();
            assert_eq!(r, 1);
        }
        q.close()?;
        Ok(())
    }

    /// # Panics
    /// Runner manager can panic on close.
    /// Each runner can panic.
    /// Sending and receiving the messages can panic.
    #[test]
    fn pool_manual_close() -> Result<(), Box<dyn std::error::Error>> {
        use supera::oneshot_pool::OneShotPoolAPI;
        const COUNT: usize = 2_500;
        let q = unsafe { OneShotPoolAPI::<MathAction, 3>::new() };
        for _ in 0..COUNT {
            let ma = MathAction::Sub(2, 1);
            let mr = q.send(ma).unwrap();
            let r = mr.recv().unwrap();
            assert_eq!(r, 1);
        }
        for r in q.close()? {
            r?;
        }
        Ok(())
    }
}
