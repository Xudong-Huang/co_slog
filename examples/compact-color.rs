extern crate slog;
extern crate slog_term;
extern crate slog_async;
#[macro_use]
extern crate co_slog;

use slog::Drain;

mod common;

fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let log = slog::Logger::root(drain, o!("version" => "0.5"));


    info!("without logger set");

    let _guard = co_slog::set_logger(log);
    common::simulate_server();
}
