//! Common test utilities.

#![allow(dead_code)]

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Create a test socket address with a given port
pub fn test_addr(port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port)
}

/// Create a test socket address with a random IP and given port
pub fn test_addr_with_ip(ip: [u8; 4], port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3])), port)
}

/// Generate a random port number for testing
pub fn random_port() -> u16 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    (now % 10000 + 50000) as u16
}
