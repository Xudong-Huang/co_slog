use std::fmt;
use std::error::Error;
use std::sync::PoisonError;
use may::sync::{Mutex, MutexGuard};
use slog::{Drain, Record, OwnedKVList};

/// Error returned by `Mutex<D : Drain>`
#[derive(Clone)]
pub enum MutexDrainError<D: Drain> {
    /// Error acquiring mutex
    Mutex,
    /// Error returned by drain
    Drain(D::Err),
}

impl<D> fmt::Debug for MutexDrainError<D>
where
    D: Drain,
    D::Err: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            MutexDrainError::Mutex => write!(f, "MutexDrainError::Mutex"),
            MutexDrainError::Drain(ref e) => e.fmt(f),
        }
    }
}

impl<D> Error for MutexDrainError<D>
where
    D: Drain,
    D::Err: fmt::Debug + fmt::Display + Error,
{
    fn description(&self) -> &str {
        match *self {
            MutexDrainError::Mutex => "Mutex acquire failed",
            MutexDrainError::Drain(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            MutexDrainError::Mutex => None,
            MutexDrainError::Drain(ref e) => Some(e),
        }
    }
}

impl<'a, D: Drain> From<PoisonError<MutexGuard<'a, D>>> for MutexDrainError<D> {
    fn from(_: PoisonError<MutexGuard<'a, D>>) -> MutexDrainError<D> {
        MutexDrainError::Mutex
    }
}

impl<D: Drain> fmt::Display for MutexDrainError<D>
where
    D::Err: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            MutexDrainError::Mutex => write!(f, "MutexError"),
            MutexDrainError::Drain(ref e) => write!(f, "{}", e),
        }
    }
}

/// coroutine mutex based logger wrapper
pub struct MutexDrain<D: Drain> {
    drain: Mutex<D>,
}

impl<D: Drain> MutexDrain<D> {
    /// wrap a normal Drain to MutexDrain
    pub fn new(d: D) -> Self {
        MutexDrain { drain: Mutex::new(d) }
    }
}

impl<D: Drain> Drain for MutexDrain<D> {
    type Ok = D::Ok;
    type Err = MutexDrainError<D>;
    fn log(&self, record: &Record, logger_values: &OwnedKVList) -> Result<Self::Ok, Self::Err> {
        self.drain.lock()?.log(record, logger_values).map_err(
            MutexDrainError::Drain,
        )
    }
}
