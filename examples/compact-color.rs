#[macro_use]
extern crate co_slog;

mod common;

fn main() {
    info!("log with default logger"; "asdfasdf" => 100, "hhee" => r#"{a: 100, b: "adfads" }"#);
    error!("bomb!");

    let log = co_slog::logger().new(o!("version" => "0.5"));
    let _guard = co_slog::set_logger(log);
    common::simulate_server();
}
