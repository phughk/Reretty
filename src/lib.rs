use socket2::{Domain, Protocol, SockAddr, Type};
use std::net::{Ipv4Addr, SocketAddrV4};

mod error;
mod handler;
mod handler_two;
mod pipeline;
#[cfg(test)]
mod test;

pub fn create() {
    let socket = socket2::Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP)).unwrap();
    socket
        .bind(&SockAddr::from(SocketAddrV4::new(
            Ipv4Addr::from([127, 0, 0, 1]),
            8080,
        )))
        .unwrap();
}
