extern crate may;
extern crate slog;
#[macro_use]
extern crate co_slog;

mod common;

use slog::Drain;
use co_slog::Async;

fn main() {
    info!("log with default logger"; "asdfasdf" => 100, "hhee" => r#"{a: 100, b: "adfads" }"#);
    error!("bomb!");

    let log = co_slog::logger();
    let log = Async::new(log.fuse()).build();
    let log = slog::Logger::root(log.fuse(), o!("version" => "0.5"));
    let _guard = co_slog::set_logger(log);
    common::simulate_server();
}
