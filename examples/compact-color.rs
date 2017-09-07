#[macro_use]
extern crate may;
#[macro_use]
extern crate slog;
extern crate slog_json;
extern crate slog_term;
extern crate serde_json;
#[macro_use]
extern crate co_slog;
#[macro_use]
extern crate serde_derive;

mod common;

use slog::Drain;
use co_slog::{AsyncDrain, EnvDrain};

#[allow(dead_code)]
fn server_1() {
    common::simulate_server();
}

#[allow(dead_code)]
fn server_2() {
    use slog::*;

    #[derive(Debug)]
    struct Foo;
    impl Value for Foo {
        fn serialize(&self, _record: &Record, key: Key, serializer: &mut Serializer) -> Result {
            serializer.emit_arguments(key, &format_args!("{:?}", self))
        }
    }

    let log = co_slog::logger();
    let log = AsyncDrain::new(log.fuse()).build();
    let log = slog::Logger::root(log.fuse(), o!("version" => Foo));
    let _guard = co_slog::set_logger(log);
    common::simulate_server();
}

#[allow(dead_code)]
fn server_3() {
    let decrator = slog_term::TermDecorator::new().stderr().build();
    let drain = slog_term::CompactFormat::new(decrator).build();
    let drain = EnvDrain::new(drain.fuse()).build();
    let drain = AsyncDrain::new(drain.fuse()).build();
    let log = slog::Logger::root(drain.fuse(), o!("version" => "0.5"));
    let _guard = co_slog::set_logger(log);
    common::simulate_server();
}

#[allow(dead_code)]
fn server_4() {
    use std::fs::File;
    use slog::*;

    #[derive(Serialize)]
    struct Foo {
        a: u32,
        b: String,
    };
    impl Value for Foo {
        fn serialize(&self, _record: &Record, key: Key, serializer: &mut Serializer) -> Result {
            serializer.emit_arguments(
                key,
                &format_args!("{}", serde_json::to_string(self).unwrap()),
            )
        }
    }

    // create the json file logger
    let file = File::create(format!("/tmp/test.log")).expect("failed to create test log");
    let d1 = slog::LevelFilter::new(slog_json::Json::default(file).fuse(), slog::Level::Info);

    // create the stderr logger
    let decrator = slog_term::TermDecorator::new().stderr().build();
    let drain = slog_term::CompactFormat::new(decrator).build();
    let d2 = EnvDrain::new(drain.fuse()).build();

    // create the duplicate logger
    let drain = slog::Duplicate::new(d1.fuse(), d2.fuse());
    let drain = AsyncDrain::new(drain.fuse()).build();

    // create the global logger
    let log = slog::Logger::root(
        drain.fuse(),
        o!("module" => slog::FnValue(|info| info.module()),
           "data" => Foo {a: 10, b: String::from("hello")}),
    );
    co_slog::set_global_logger(log);
    // info!("log init done");

    common::simulate_server();
}

fn main() {
    info!("log with default logger"; "asdfasdf" => 100, "hhee" => r#"{a: 100, b: "adfads" }"#);
    error!("bomb!");

    // co_slog::set_global_logger(slog::Logger::root(slog::Discard, o!()));

    server_4();
}
