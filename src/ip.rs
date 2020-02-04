use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6};
use std::str::FromStr;

use anyhow::Result;
use tokio::net::UdpSocket;

use lazy_static::lazy_static;

lazy_static! {
static ref CF_IPV4_DNS_IP: Ipv4Addr = Ipv4Addr::from_str("1.0.0.1").unwrap();
static ref CF_IPV6_DNS_IP: Ipv6Addr = Ipv6Addr::from_str("2606:4700:4700::1111").unwrap();
}

pub async fn get_local_ipv6_ip() -> Result<Ipv6Addr> {
    let udp_socket = UdpSocket::bind(SocketAddrV6::from_str("[::]:0").unwrap()).await?;

    udp_socket.connect((*CF_IPV6_DNS_IP, 53)).await?;

    if let IpAddr::V6(ipv6) = udp_socket.local_addr()?.ip() {
        return Ok(ipv6);
    }

    unreachable!()
}

pub async fn get_local_ipv4_ip() -> Result<Ipv4Addr> {
    let udp_socket = UdpSocket::bind(SocketAddrV4::from_str("0.0.0.0:0").unwrap()).await?;

    udp_socket.connect((*CF_IPV4_DNS_IP, 53)).await?;

    if let IpAddr::V4(ipv4) = udp_socket.local_addr()?.ip() {
        return Ok(ipv4);
    }

    unreachable!()
}