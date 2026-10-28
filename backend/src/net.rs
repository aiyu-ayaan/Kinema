//! Where a request comes from, for LAN-only profiles. The socket peer is the client unless
//! it is one of the configured `[network] trusted_proxies`; only then is `X-Forwarded-For`
//! read. A request that carries forwarding headers from anyone else counts as remote, so
//! a reverse proxy on the same machine (peer 127.0.0.1) can never make the internet look
//! like the living room. Unknown always means remote.

use crate::state::AppState;
use axum::async_trait;
use axum::extract::{ConnectInfo, FromRequestParts};
use axum::http::HeaderMap;
use axum::http::request::Parts;
use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

/// Private, loopback and link-local addresses: the household's own network.
/// CGNAT (100.64/10) is shared with other ISP customers, so it is not LAN; nor is a
/// global IPv6 address, even one on the same home network (fails closed: PIN asked).
pub fn is_lan(ip: IpAddr) -> bool {
    match canonical(ip) {
        IpAddr::V4(v4) => is_lan_v4(v4),
        IpAddr::V6(v6) => is_lan_v6(v6),
    }
}

fn is_lan_v4(ip: Ipv4Addr) -> bool {
    ip.is_private() || ip.is_loopback() || ip.is_link_local()
}

fn is_lan_v6(ip: Ipv6Addr) -> bool {
    ip.is_loopback() || ip.is_unique_local() || ip.is_unicast_link_local()
}

/// `::ffff:a.b.c.d` (a dual-stack socket's view of an IPv4 client) as plain IPv4.
fn canonical(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => v6.to_ipv4_mapped().map_or(ip, IpAddr::V4),
        v4 => v4,
    }
}

/// An address or a network in CIDR form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cidr {
    net: IpAddr,
    prefix: u8,
}

impl Cidr {
    pub fn parse(s: &str) -> Option<Self> {
        let (addr, prefix) = match s.trim().split_once('/') {
            Some((a, p)) => (a, Some(p.parse::<u8>().ok()?)),
            None => (s.trim(), None),
        };
        let net = canonical(addr.parse::<IpAddr>().ok()?);
        let max = if net.is_ipv4() { 32 } else { 128 };
        let prefix = prefix.unwrap_or(max);
        (prefix <= max).then_some(Self { net, prefix })
    }

    pub fn contains(&self, ip: IpAddr) -> bool {
        match (self.net, canonical(ip)) {
            (IpAddr::V4(n), IpAddr::V4(a)) => prefix_eq(
                u128::from(u32::from(n)),
                u128::from(u32::from(a)),
                32,
                self.prefix,
            ),
            (IpAddr::V6(n), IpAddr::V6(a)) => prefix_eq(n.into(), a.into(), 128, self.prefix),
            _ => false,
        }
    }
}

fn prefix_eq(a: u128, b: u128, bits: u8, prefix: u8) -> bool {
    if prefix == 0 {
        return true;
    }
    let shift = u32::from(bits - prefix);
    (a >> shift) == (b >> shift)
}

pub fn parse_proxies(list: &[String]) -> Vec<Cidr> {
    list.iter().filter_map(|s| Cidr::parse(s)).collect()
}

const FORWARD_HEADERS: &[&str] = &["x-forwarded-for", "forwarded", "x-real-ip"];

/// The client's address, or None when it cannot be trusted.
pub fn client_ip(peer: Option<IpAddr>, headers: &HeaderMap, trusted: &[Cidr]) -> Option<IpAddr> {
    let peer = canonical(peer?);
    let forwarded = FORWARD_HEADERS.iter().any(|h| headers.contains_key(*h));
    let is_trusted = |ip: IpAddr| trusted.iter().any(|c| c.contains(ip));
    if !is_trusted(peer) {
        return (!forwarded).then_some(peer);
    }
    // Walk right to left: the rightmost entry was added by our proxy, entries further left
    // can be forged by the client. The first one that is not a trusted proxy is the client.
    let hops: Vec<&str> = headers
        .get_all("x-forwarded-for")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(','))
        .map(str::trim)
        .collect();
    for hop in hops.iter().rev() {
        let ip = canonical(hop.parse::<IpAddr>().ok()?);
        if !is_trusted(ip) {
            return Some(ip);
        }
    }
    // A trusted proxy that did not say who the client is: unknown.
    None
}

/// The request's origin as far as LAN-only rules are concerned.
#[derive(Debug, Clone, Copy)]
pub struct ClientOrigin {
    pub lan: bool,
}

pub fn origin(parts: &Parts, state: &AppState) -> ClientOrigin {
    let peer = parts
        .extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0.ip());
    let trusted = parse_proxies(&state.config.network.trusted_proxies);
    ClientOrigin {
        lan: client_ip(peer, &parts.headers, &trusted).is_some_and(is_lan),
    }
}

/// The client's address under the same trust rules as [`ClientOrigin`]; None when it cannot be trusted.
#[derive(Debug, Clone, Copy)]
pub struct ClientAddr(pub Option<IpAddr>);

#[async_trait]
impl FromRequestParts<AppState> for ClientAddr {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Infallible> {
        let peer = parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|c| c.0.ip());
        let trusted = parse_proxies(&state.config.network.trusted_proxies);
        Ok(Self(client_ip(peer, &parts.headers, &trusted)))
    }
}

#[async_trait]
impl FromRequestParts<AppState> for ClientOrigin {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Infallible> {
        Ok(origin(parts, state))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn ipv4_classes() {
        for lan in [
            "10.1.2.3",
            "172.16.0.1",
            "172.31.255.255",
            "192.168.1.20",
            "127.0.0.1",
            "169.254.10.10",
        ] {
            assert!(is_lan(ip(lan)), "{lan}");
        }
        for wan in [
            "8.8.8.8",
            "172.32.0.1",
            "100.64.0.1",
            "192.169.0.1",
            "0.0.0.0",
            "255.255.255.255",
        ] {
            assert!(!is_lan(ip(wan)), "{wan}");
        }
    }

    #[test]
    fn ipv6_classes() {
        assert!(is_lan(ip("::1")));
        assert!(is_lan(ip("fd12:3456:789a::1")), "ULA");
        assert!(is_lan(ip("fc00::1")), "ULA");
        assert!(is_lan(ip("fe80::1")), "link-local");
        assert!(!is_lan(ip("2001:db8::1")), "global");
        assert!(!is_lan(ip("2a00:1450:4001::1")), "global");
        assert!(!is_lan(ip("fec0::1")), "deprecated site-local");
        assert!(!is_lan(ip("::")), "unspecified");
        assert!(!is_lan(ip("64:ff9b::808:808")), "NAT64 of a public address");
    }

    #[test]
    fn ipv4_mapped() {
        assert!(is_lan(ip("::ffff:192.168.1.5")));
        assert!(is_lan(ip("::ffff:127.0.0.1")));
        assert!(!is_lan(ip("::ffff:8.8.8.8")));
    }

    #[test]
    fn cidr() {
        let c = Cidr::parse("10.0.0.0/8").unwrap();
        assert!(c.contains(ip("10.200.1.1")));
        assert!(c.contains(ip("::ffff:10.0.0.9")));
        assert!(!c.contains(ip("11.0.0.1")));
        assert!(Cidr::parse("127.0.0.1").unwrap().contains(ip("127.0.0.1")));
        assert!(!Cidr::parse("127.0.0.1").unwrap().contains(ip("127.0.0.2")));
        let v6 = Cidr::parse("fd00::/8").unwrap();
        assert!(v6.contains(ip("fd12::1")));
        assert!(!v6.contains(ip("fe80::1")));
        assert!(Cidr::parse("0.0.0.0/0").unwrap().contains(ip("8.8.8.8")));
        assert!(Cidr::parse("10.0.0.0/33").is_none());
        assert!(Cidr::parse("nope").is_none());
    }

    fn headers(xff: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert("x-forwarded-for", HeaderValue::from_str(xff).unwrap());
        h
    }

    #[test]
    fn peer_is_the_client_without_proxies() {
        let none = HeaderMap::new();
        assert_eq!(
            client_ip(Some(ip("192.168.1.4")), &none, &[]),
            Some(ip("192.168.1.4"))
        );
        assert_eq!(client_ip(None, &none, &[]), None);
        // A forged header from an untrusted peer is ignored and makes the request remote,
        // even when the peer itself is local (a proxy on the same machine).
        assert_eq!(
            client_ip(Some(ip("8.8.8.8")), &headers("192.168.1.4"), &[]),
            None
        );
        assert_eq!(
            client_ip(Some(ip("127.0.0.1")), &headers("8.8.8.8"), &[]),
            None
        );
        let mut real_ip = HeaderMap::new();
        real_ip.insert("x-real-ip", HeaderValue::from_static("192.168.1.4"));
        assert_eq!(client_ip(Some(ip("127.0.0.1")), &real_ip, &[]), None);
    }

    #[test]
    fn trusted_proxy_forwards() {
        let trusted = parse_proxies(&["127.0.0.1".into(), "10.0.0.0/8".into()]);
        let peer = Some(ip("127.0.0.1"));
        assert_eq!(
            client_ip(peer, &headers("203.0.113.9"), &trusted),
            Some(ip("203.0.113.9"))
        );
        // The client prepended a LAN address; the proxy appended the real one.
        assert_eq!(
            client_ip(peer, &headers("192.168.1.4, 203.0.113.9"), &trusted),
            Some(ip("203.0.113.9"))
        );
        // Chained trusted proxies are skipped.
        assert_eq!(
            client_ip(peer, &headers("192.168.1.4, 10.0.0.2"), &trusted),
            Some(ip("192.168.1.4"))
        );
        // Missing or garbage header from a trusted proxy: unknown.
        assert_eq!(client_ip(peer, &HeaderMap::new(), &trusted), None);
        assert_eq!(client_ip(peer, &headers("bogus"), &trusted), None);
        assert_eq!(client_ip(peer, &headers("10.0.0.3"), &trusted), None);
    }
}
