mod api;
#[cfg(feature = "client")]
mod client;
#[cfg(feature = "server")]
mod server;

#[cfg(not(any(feature = "client", feature = "server")))]
compile_error!("You must enable at least the client or server feature!");

fn main() {
    #[cfg(feature = "server")]
    {
        use std::net::{Ipv4Addr, SocketAddr, ToSocketAddrs};

        let args: Vec<_> = std::env::args().collect();

        if cfg!(not(feature = "client"))
            || args.get(1).is_some_and(|x| x == "--server" || x == "-s")
        {
            let addr = if let Some((_, port)) = args
                .iter()
                .filter_map(|s| s.split_once('='))
                .find(|&arg| arg.0 == "-p" || arg.0 == "--port")
            {
                port.to_socket_addrs()
                    .map(|mut p| p.next())
                    .unwrap_or_default()
                    .expect("invalid port or domain")
            } else {
                SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 1812)
            };
            server::start_server(addr);
            #[cfg(feature = "client")]
            return;
        }
    }

    #[cfg(feature = "client")]
    client::start_client();
}
