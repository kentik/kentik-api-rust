use backoff::{backoff::Backoff, ExponentialBackoff};
use futures_retry::{
    ErrorHandler,
    FutureFactory,
    FutureRetry,
    RetryPolicy
};
use crate::Error;

#[derive(Default)]
pub struct Retry {
    backoff: ExponentialBackoff,
    retries: usize,
}

pub fn retry<F: FutureFactory>(factory: F, retries: usize) -> FutureRetry<F, Retry> {
    FutureRetry::new(factory, Retry {
        backoff: ExponentialBackoff::default(),
        retries: retries,
    })
}

impl ErrorHandler<Error> for Retry {
    type OutError = Error;

    fn handle(&mut self, attempt: usize, e: Error) -> RetryPolicy<Error> {
        if attempt > self.retries {
            return RetryPolicy::ForwardError(e);
        }

        let e = match e.into_backoff() {
            backoff::Error::Transient{err, ..} => err,
            backoff::Error::Permanent(err)     => return RetryPolicy::ForwardError(err),
        };

        match self.backoff.next_backoff() {
            Some(d) => RetryPolicy::WaitRetry(d),
            None    => RetryPolicy::ForwardError(e),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Error::*;
    use Decision::*;

    #[test]
    fn zero_retries() {
        let mut retry = retry(0);
        assert_eq!(Stop, retry.handle(1, Status(503)).into());
    }

    #[test]
    fn one_retry() {
        let mut retry = retry(1);
        assert_eq!(Wait, retry.handle(1, Status(503)).into());
        assert_eq!(Stop, retry.handle(2, Status(503)).into());
    }

    #[test]
    fn two_retries() {
        let mut retry = retry(2);
        assert_eq!(Wait, retry.handle(1, Status(503)).into());
        assert_eq!(Wait, retry.handle(2, Status(503)).into());
        assert_eq!(Stop, retry.handle(3, Status(503)).into());
    }

    #[test]
    fn ensure_retry() {
        assert_eq!(Wait, retry(1).handle(0, App(String::new(), 500)).into());
        assert_eq!(Wait, retry(1).handle(0, Status(500)).into());
        assert_eq!(Wait, retry(1).handle(0, Timeout).into());
        assert_eq!(Wait, retry(1).handle(0, Other(String::new())).into());
    }

    #[test]
    fn ensure_no_retry() {
        assert_eq!(Stop, retry(1).handle(0, Auth).into());
        assert_eq!(Stop, retry(1).handle(0, App(String::new(), 400)).into());
        assert_eq!(Stop, retry(1).handle(0, Status(400)).into());
        assert_eq!(Stop, retry(1).handle(0, Empty).into());
    }

    fn retry(retries: usize) -> Retry {
        Retry{retries, ..Default::default()}
    }

    #[derive(Eq, PartialEq, Debug)]
    enum Decision {
        Wait,
        Stop,
        Repeat,
    }

    impl<E> From<RetryPolicy<E>> for Decision {
        fn from(p: RetryPolicy<E>) -> Self {
            match p {
                RetryPolicy::WaitRetry(_)    => Decision::Wait,
                RetryPolicy::ForwardError(_) => Decision::Stop,
                RetryPolicy::Repeat          => Decision::Repeat,
            }
        }
    }
}
