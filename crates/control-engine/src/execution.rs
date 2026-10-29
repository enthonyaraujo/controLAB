//! Request-local cooperative deadline/cancellation; scopes restore prior state.
use crate::Result;
use std::{
    cell::RefCell,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
struct Context {
    stop: Arc<AtomicBool>,
    deadline: Instant,
}
thread_local! {static ACTIVE:RefCell<Option<Context>>=const{RefCell::new(None)};}
pub(crate) fn run<T>(
    stop: Arc<AtomicBool>,
    timeout: Duration,
    f: impl FnOnce() -> Result<T>,
) -> Result<T> {
    struct Restore(Option<Context>);
    impl Drop for Restore {
        fn drop(&mut self) {
            ACTIVE.with(|s| {
                s.replace(self.0.take());
            });
        }
    }
    let _restore = Restore(ACTIVE.with(|s| {
        s.replace(Some(Context {
            stop,
            deadline: Instant::now() + timeout,
        }))
    }));
    checkpoint()?;
    let value = f()?;
    checkpoint()?;
    Ok(value)
}
pub fn checkpoint() -> Result<()> {
    ACTIVE.with(|s| {
        let context = s.borrow();
        if let Some(c) = context.as_ref() {
            if c.stop.load(Ordering::Relaxed) {
                return Err("Aplicativo encerrando.".into());
            }
            if Instant::now() >= c.deadline {
                return Err("Tempo de cálculo excedido.".into());
            }
        }
        Ok(())
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancellation_and_deadlines_restore_request_context() {
        assert!(
            run(
                Arc::new(AtomicBool::new(true)),
                Duration::from_secs(1),
                || Ok(())
            )
            .is_err()
        );
        assert!(checkpoint().is_ok());
        assert!(run(Arc::new(AtomicBool::new(false)), Duration::ZERO, || Ok(())).is_err());
        assert!(checkpoint().is_ok());
        run(
            Arc::new(AtomicBool::new(false)),
            Duration::from_secs(1),
            || {
                assert!(
                    run(
                        Arc::new(AtomicBool::new(true)),
                        Duration::from_secs(1),
                        || Ok(())
                    )
                    .is_err()
                );
                checkpoint()
            },
        )
        .unwrap();
    }
}
