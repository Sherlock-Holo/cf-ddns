use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use anyhow::Result;
use tokio::net::UdpSocket;

pub async fn get_local_ipv6_ip() -> Result<Ipv6Addr> {
    let udp_socket = UdpSocket::bind("[::]:0").await?;

    udp_socket
        .connect((Ipv6Addr::from_str("2606:4700:4700::1111")?, 53))
        .await?;

    if let IpAddr::V6(ipv6) = udp_socket.local_addr()?.ip() {
        return Ok(ipv6);
    }

    unreachable!()
}

pub async fn get_local_ipv4_ip() -> Result<Ipv4Addr> {
    let udp_socket = UdpSocket::bind("0.0.0.0:0").await?;

    udp_socket
        .connect((Ipv4Addr::from_str("1.0.0.1")?, 53))
        .await?;

    if let IpAddr::V4(ipv4) = udp_socket.local_addr()?.ip() {
        return Ok(ipv4);
    }

    unreachable!()
}
