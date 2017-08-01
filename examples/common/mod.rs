use co_slog;

pub fn simulate_server() {

    let log = co_slog::logger();

    let server = log.new(o!("host" => "localhost", "port" => "8080"));
    let peer1 = server.new(o!("peer_addr" => "8.8.8.8", "port" => "18230"));
    let peer2 = server.new(o!("peer_addr" => "82.9.9.9", "port" => "42381"));

    co_slog::set_logger(server);
    info!("starting");
    info!("listening");

    {
        let _log = co_slog::set_logger(peer2.clone());
        debug!("connected");
        debug!("message received"; "length" => 2);
    }

    {
        let _log = co_slog::set_logger(peer1.clone());
        debug!("connected");
    }

    {
        let _log = co_slog::set_logger(peer2);
        warn!("weak encryption requested"; "algo" => "xor");
        debug!("response sent"; "length" => 8);
        debug!("disconnected");
    }

    {
        let _log = co_slog::set_logger(peer1);
        debug!("message received"; "length" => 2);
        debug!("response sent"; "length" => 8);
        debug!("disconnected");
    }

    crit!("internal error");
    info!("exit");

}
