use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

mod api;
mod client;
mod server;

extern crate bevy_slinet;

fn main() {
    let args: Vec<_> = std::env::args().collect();

    if args.get(1).is_some_and(|x| x == "--server" || x == "-s") {
        let addr = if let Some(port) = args
            .iter()
            .find(|&arg| arg.starts_with("-p=") || arg.starts_with("--port="))
        {
            match port.parse() {
                Ok(x) => x,
                Err(_) if port.parse::<Ipv4Addr>().is_ok() => {
                    SocketAddr::V4(SocketAddrV4::new(port.parse().unwrap(), 1812))
                }
                Err(_) => panic!("invalid port, got: {:?}", port),
            }
        } else {
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 1812))
        };
        server::start_server(addr);
        return;
    }

    client::start_client();
}
