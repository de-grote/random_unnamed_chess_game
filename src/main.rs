mod api;
#[cfg(feature = "client")]
mod client;
#[cfg(feature = "server")]
mod server;

#[cfg(not(any(feature = "client", feature = "server")))] compile_error!("You must enable at least the client or server feature!");

fn main() {
    #[cfg(feature = "server")]
    {
        use std::net::{Ipv4Addr, SocketAddr};

        let args: Vec<_> = std::env::args().collect();

        if cfg!(not(feature = "client")) || args.get(1).is_some_and(|x| x == "--server" || x == "-s") {
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
            #[cfg(feature = "client")]
            return;
        }
    }

    #[cfg(feature = "client")]
    client::start_client();
}
