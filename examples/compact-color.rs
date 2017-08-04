#[macro_use]
extern crate may;
extern crate slog;
extern crate slog_term;
#[macro_use]
extern crate co_slog;

mod common;

use slog::Drain;

#[allow(dead_code)]
fn server_1() {
    common::simulate_server();
}

#[allow(dead_code)]
fn server_2() {
    use co_slog::AsyncDrain;

    let log = co_slog::logger();
    let log = AsyncDrain::new(log.fuse()).build();
    let log = slog::Logger::root(log.fuse(), o!("version" => "0.5"));
    let _guard = co_slog::set_logger(log);
    common::simulate_server();
}

#[allow(dead_code)]
fn server_3() {
    use co_slog::{AsyncDrain, EnvDrain};

    let decrator = slog_term::TermDecorator::new().stderr().build();
    let drain = slog_term::CompactFormat::new(decrator).build();
    let drain = EnvDrain::new(drain.fuse()).build();
    let drain = AsyncDrain::new(drain.fuse()).build();
    let log = slog::Logger::root(drain.fuse(), o!("version" => "0.5"));
    let _guard = co_slog::set_logger(log);
    common::simulate_server();
}

fn main() {
    info!("log with default logger"; "asdfasdf" => 100, "hhee" => r#"{a: 100, b: "adfads" }"#);
    error!("bomb!");

    // co_slog::set_global_logger(slog::Logger::root(slog::Discard, o!()));

    server_3();
}
