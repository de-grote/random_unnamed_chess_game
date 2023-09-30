use std::net::{Ipv4Addr, SocketAddr};

mod api;
mod client;
mod server;

fn main() {
    let args: Vec<_> = std::env::args().collect();

    if args.get(1).is_some_and(|x| x == "--server" || x == "-s") {
        let addr = if let Some(port) = args
            .iter()
            .find(|&arg| arg.starts_with("-p=") || arg.starts_with("--port="))
        {
            port.parse().unwrap_or_else(|e| {
                SocketAddr::new(
                    port.parse()
                        .unwrap_or_else(|_| panic!("invalid port, got: {:?}, {}", port, e)),
                    1812,
                )
            })
        } else {
            SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 1812)
        };
        server::start_server(addr);
        return;
    }

    client::start_client();
}
