use std::{
    io::ErrorKind,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener},
};

pub fn is_localhost_tcp_port_available(port: u16) -> bool {
    localhost_bind_available(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
        && localhost_bind_available(IpAddr::V6(Ipv6Addr::LOCALHOST), port)
}

fn localhost_bind_available(address: IpAddr, port: u16) -> bool {
    match TcpListener::bind(SocketAddr::from((address, port))) {
        Ok(listener) => {
            drop(listener);
            true
        }
        Err(error) => matches!(
            error.kind(),
            ErrorKind::AddrNotAvailable | ErrorKind::Unsupported
        ),
    }
}
