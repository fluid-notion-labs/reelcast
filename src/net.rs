/// Returns the primary outbound LAN IP via UDP socket trick.
/// No extra deps, no interface enumeration needed.
pub fn local_ip() -> Option<std::net::IpAddr> {
    let sock = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    sock.connect("8.8.8.8:80").ok()?;
    let ip = sock.local_addr().ok()?.ip();
    if ip.is_loopback() { None } else { Some(ip) }
}
