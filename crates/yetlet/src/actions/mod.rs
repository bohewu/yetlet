//! Semantic commands and explicitly injectable action executors.
//!
//! UI events never cross this abstraction. Browser code turns an event into a
//! typed command and chooses a local or transport executor; no persistent circuit.
//! Each executor chooses its error type. Serialization and HTTP errors belong
//! to the separately selected actions-http capability.

/// Local-only commands do not need serialization. A transport adds that bound
/// explicitly at its own boundary.
pub trait Command {
    type Response;
}

/// A static-dispatch command boundary, independent of DOM event types.
/// Browser-local closures/futures are intentionally not required to be Send.
#[allow(async_fn_in_trait)]
pub trait ActionExecutor<C: Command> {
    /// The domain or transport error chosen by this executor.
    type Error;

    async fn execute(&self, command: C) -> Result<C::Response, Self::Error>;
}

/// Local actions can use any injected pure or stateful command handler.
/// The caller controls its state lifetime and error type; no framework session
/// or transport mapping is created.
#[derive(Clone)]
pub struct LocalExecutor<F> {
    handler: F,
}

impl<F> LocalExecutor<F> {
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

impl<C, F, E> ActionExecutor<C> for LocalExecutor<F>
where
    C: Command,
    F: Fn(C) -> Result<C::Response, E>,
{
    type Error = E;

    async fn execute(&self, command: C) -> Result<C::Response, Self::Error> {
        (self.handler)(command)
    }
}

#[cfg(test)]
mod tests {
    use super::{ActionExecutor, Command, LocalExecutor};
    use std::{
        cell::Cell,
        future::{Future, poll_fn},
        pin::pin,
        rc::Rc,
        task::{Context, Poll, Waker},
    };

    struct Increment(u32);

    impl Command for Increment {
        type Response = u32;
    }

    #[derive(Debug, PartialEq, Eq)]
    struct LimitReached(Rc<str>);

    #[test]
    fn local_executor_retains_caller_state_and_domain_error() {
        let count = Rc::new(Cell::new(0));
        let reason: Rc<str> = Rc::from("application limit");
        let executor = LocalExecutor::new({
            let count = Rc::clone(&count);
            let reason = Rc::clone(&reason);
            move |command: Increment| {
                let next = count.get() + command.0;
                if next > 3 {
                    return Err(LimitReached(Rc::clone(&reason)));
                }
                count.set(next);
                Ok(next)
            }
        });
        let mut context = Context::from_waker(Waker::noop());
        assert_eq!(
            pin!(executor.execute(Increment(2)))
                .as_mut()
                .poll(&mut context),
            Poll::Ready(Ok(2))
        );
        assert_eq!(
            pin!(executor.execute(Increment(2)))
                .as_mut()
                .poll(&mut context),
            Poll::Ready(Err(LimitReached(reason)))
        );
        assert_eq!(count.get(), 2);
    }

    #[test]
    fn custom_executor_can_suspend_with_non_send_state_and_error() {
        struct LocalCounter(Rc<Cell<u32>>);

        impl ActionExecutor<Increment> for LocalCounter {
            type Error = LimitReached;

            async fn execute(&self, command: Increment) -> Result<u32, Self::Error> {
                let count = Rc::clone(&self.0);
                let mut suspended = false;
                poll_fn(|context| {
                    if suspended {
                        Poll::Ready(())
                    } else {
                        suspended = true;
                        context.waker().wake_by_ref();
                        Poll::Pending
                    }
                })
                .await;
                count.set(count.get() + command.0);
                Ok(count.get())
            }
        }

        let count = Rc::new(Cell::new(1));
        let executor = LocalCounter(Rc::clone(&count));
        let mut future = pin!(executor.execute(Increment(2)));
        let mut context = Context::from_waker(Waker::noop());
        assert_eq!(future.as_mut().poll(&mut context), Poll::Pending);
        assert_eq!(count.get(), 1);
        assert_eq!(future.as_mut().poll(&mut context), Poll::Ready(Ok(3)));
        assert_eq!(count.get(), 3);
    }
}
