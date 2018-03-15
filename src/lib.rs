//! Logging scopes for slog-rs
//!
//! Logging scopes are convenience functionality for slog-rs to free user from manually passing
//! `Logger` objects around.
//!
//! Set of macros is also provided as an alternative to original `slog` crate macros, for logging
//! directly to `Logger` of the current logging scope.
//!
//! ```
//! #[macro_use(slog_o, slog_info, slog_log, slog_record, slog_record_static, slog_b, slog_kv)]
//! extern crate slog;
//! #[macro_use]
//! extern crate co_slog;
//! extern crate slog_term;
//!
//! use slog::Drain;
//!
//! fn foo() {
//!     slog_info!(co_slog::logger(), "foo");
//!     info!("foo"); // Same as above, but more ergonomic and a bit faster
//!                   // since it uses `with_logger`
//! }
//!
//! fn main() {
//!     let plain = slog_term::PlainSyncDecorator::new(std::io::stdout());
//!     let log = slog::Logger::root(
//!         slog_term::FullFormat::new(plain)
//!         .build().fuse(), slog_o!()
//!     );
//!
//!     // Make sure to save the guard, see documentation for more information
//!     let _guard = co_slog::set_logger(log.new(slog_o!("scope" => "1")));
//!     foo();
//! }
#![warn(missing_docs)]

#[macro_use(coroutine_local)]
extern crate may;
#[macro_use]
extern crate slog;
extern crate regex;
extern crate take_mut;
extern crate slog_term;
extern crate crossbeam;
#[macro_use]
extern crate lazy_static;

mod env_drain;
mod mutex_drain;
mod async_drain;

use slog::Logger;
use std::sync::Arc;
use std::cell::RefCell;
use crossbeam::sync::ArcCell;

pub use slog::Drain;
pub use env_drain::EnvDrain;
pub use async_drain::AsyncDrain;
pub use mutex_drain::MutexDrain;

/// Log a critical level message using current scope logger
#[macro_export]
macro_rules! crit( ($($args:tt)+) => {
    $crate::with_logger(|logger| slog_crit![logger, $($args)+])
};);
/// Log a error level message using current scope logger
#[macro_export]
macro_rules! error( ($($args:tt)+) => {
    $crate::with_logger(|logger| slog_error![logger, $($args)+])
};);
/// Log a warning level message using current scope logger
#[macro_export]
macro_rules! warn( ($($args:tt)+) => {
    $crate::with_logger(|logger| slog_warn![logger, $($args)+])
};);
/// Log a info level message using current scope logger
#[macro_export]
macro_rules! info( ($($args:tt)+) => {
    $crate::with_logger(|logger| slog_info![logger, $($args)+])
};);
/// Log a debug level message using current scope logger
#[macro_export]
macro_rules! debug( ($($args:tt)+) => {
    $crate::with_logger(|logger| slog_debug![logger, $($args)+])
};);
/// Log a trace level message using current scope logger
#[macro_export]
macro_rules! trace( ($($args:tt)+) => {
    $crate::with_logger(|logger| slog_trace![logger, $($args)+])
};);

/// Use a default `EnvLogger` as global logging drain
lazy_static! {
    static ref ENV_LOGGER : slog::Logger = {
        env_drain::stderr_logger()
    };

    static ref GLOBAL_LOGGER : ArcCell<slog::Logger> = {
        ArcCell::new(Arc::new(ENV_LOGGER.clone()))
    };
}


/// On drop it will reset global logger to `ENV_LOGGER`.
struct GlobalLoggerGuard;
impl Drop for GlobalLoggerGuard {
    fn drop(&mut self) {
        GLOBAL_LOGGER.set(Arc::new(ENV_LOGGER.clone()));
    }
}

/// the global logger guard, valid for the whole life of the thread/coroutine
coroutine_local! {
    static GLOBAL_GUARD: RefCell<Option<GlobalLoggerGuard>> = {
        RefCell::new(None)
    }
}

/// Set global `Logger`
/// if the setter thread/coroutine exit,
/// the global logger would reset to default EVN_LOGGER
pub fn set_global_logger(l: slog::Logger) {
    GLOBAL_LOGGER.set(Arc::new(l));
    GLOBAL_GUARD.with(|g| *g.borrow_mut() = Some(GlobalLoggerGuard));
}

/// the logger stack infrustructure
coroutine_local! {
    static TL_SCOPES: RefCell<Vec<slog::Logger>> = {
        RefCell::new(Vec::with_capacity(8))
    }
}

/// scope logger guard, when dropped would pop it's own logger
pub struct ScopeGuard;

impl ScopeGuard {
    /// push
    fn new(logger: slog::Logger) -> Self {
        TL_SCOPES.with(|s| s.borrow_mut().push(logger));
        ScopeGuard
    }
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        TL_SCOPES.with(|s| {
            s.borrow_mut().pop().expect(
                "TL_SCOPES should contain a logger",
            );
        })
    }
}

/// push the `Logger` for the following logging scope
/// return a `ScopeGuard` when drop would automatically pop
pub fn set_logger(logger: slog::Logger) -> ScopeGuard {
    ScopeGuard::new(logger)
}

/// Access the `Logger` for the current logging scope
///
/// This function needs to clone an underlying scoped
/// `Logger`. If performance is of vital importance,
/// use `with_logger`.
pub fn logger() -> Logger {
    TL_SCOPES.with(|s| {
        let s = s.borrow();
        match s.last() {
            Some(logger) => logger.clone(),
            None => (*GLOBAL_LOGGER.get()).clone(),
        }
    })
}

/// Access the `Logger` for the current logging scope
///
/// This function doesn't have to clone the Logger
/// so it might be a bit faster.
pub fn with_logger<F, R>(f: F) -> R
where
    F: FnOnce(&Logger) -> R,
{
    TL_SCOPES.with(|s| {
        let s = s.borrow();
        match s.last() {
            Some(logger) => f(logger),
            None => f(&(*GLOBAL_LOGGER.get())),
        }
    })
}
