//! Where the LLM client may connect.
//!
//! A local model on a private address is the normal case, so private ranges stay open. What
//! is refused are the addresses that hand out credentials to whoever asks from inside: the
//! link-local range with the cloud metadata services, the metadata addresses outside that
//! range, and the Home Assistant Supervisor. The check runs on the literal address in the URL
//! and, for a host name, on the addresses it resolves to at connect time.

use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use std::future::Future;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::pin::Pin;
use std::sync::Arc;

/// Alibaba Cloud metadata. It sits in the carrier-grade NAT range, which is otherwise allowed.
const ALIBABA_METADATA: Ipv4Addr = Ipv4Addr::new(100, 100, 100, 200);
/// The Supervisor API inside the add-on network.
const SUPERVISOR: Ipv4Addr = Ipv4Addr::new(172, 30, 32, 2);
/// EC2 metadata over IPv6.
const EC2_METADATA_V6: Ipv6Addr = Ipv6Addr::new(0xfd00, 0x0ec2, 0, 0, 0, 0, 0, 0x0254);

const BLOCKED_NAMES: &[&str] = &["metadata.google.internal", "metadata", "supervisor", "hassio"];

pub(crate) fn blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => blocked_v4(v4),
        IpAddr::V6(v6) => match v6.to_ipv4_mapped() {
            Some(v4) => blocked_v4(v4),
            None => v6.is_unicast_link_local() || v6 == EC2_METADATA_V6,
        },
    }
}

fn blocked_v4(ip: Ipv4Addr) -> bool {
    ip.is_link_local() || ip == ALIBABA_METADATA || ip == SUPERVISOR
}

pub(crate) fn blocked_name(name: &str) -> bool {
    let name = name.trim_end_matches('.').to_ascii_lowercase();
    BLOCKED_NAMES.contains(&name.as_str())
}

/// The host of a base URL as written: an address is checked as an address, a name by name.
pub(crate) fn blocked_url(url: &url::Url) -> bool {
    match url.host() {
        Some(url::Host::Ipv4(ip)) => blocked_ip(IpAddr::V4(ip)),
        Some(url::Host::Ipv6(ip)) => blocked_ip(IpAddr::V6(ip)),
        Some(url::Host::Domain(name)) => blocked_name(name),
        None => true,
    }
}

type Lookup = Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output = io::Result<Vec<SocketAddr>>> + Send>> + Send + Sync>;

/// Name resolution for the LLM client. A name is refused when any address it resolves to is
/// blocked, so a public name that points inward gets no connection at all.
pub(crate) struct GuardedResolver {
    lookup: Lookup,
}

impl GuardedResolver {
    /// Resolve through the system resolver.
    pub(crate) fn system() -> Self {
        Self::with_lookup(|host| async move { Ok(tokio::net::lookup_host((host.as_str(), 0)).await?.collect()) })
    }

    pub(crate) fn with_lookup<F, Fut>(lookup: F) -> Self
    where
        F: Fn(String) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = io::Result<Vec<SocketAddr>>> + Send + 'static,
    {
        Self { lookup: Arc::new(move |host| Box::pin(lookup(host))) }
    }
}

impl Resolve for GuardedResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let lookup = self.lookup.clone();
        Box::pin(async move {
            let host = name.as_str().to_string();
            let found = lookup(host.clone()).await?;
            let admitted = admit(&host, found)?;
            Ok(Box::new(admitted.into_iter()) as Addrs)
        })
    }
}

fn admit(host: &str, found: Vec<SocketAddr>) -> io::Result<Vec<SocketAddr>> {
    if blocked_name(host) || found.iter().any(|addr| blocked_ip(addr.ip())) {
        tracing::warn!("LLM-Ziel {host} abgelehnt: der Name zeigt auf eine gesperrte Adresse");
        return Err(io::Error::new(io::ErrorKind::PermissionDenied, "llm target resolves to a blocked address"));
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(raw: &str) -> IpAddr {
        raw.parse().unwrap()
    }

    #[test]
    fn metadata_and_supervisor_addresses_are_blocked() {
        for raw in
            ["169.254.169.254", "169.254.0.1", "100.100.100.200", "172.30.32.2", "fe80::1", "fd00:ec2::254", "::ffff:169.254.169.254"]
        {
            assert!(blocked_ip(ip(raw)), "{raw}");
        }
    }

    #[test]
    fn local_models_stay_reachable() {
        // Loopback, a LAN host, another add-on, a tailnet peer, a public API.
        for raw in ["127.0.0.1", "192.168.178.15", "172.30.33.4", "100.101.102.103", "10.0.0.7", "::1", "fd12:3456::9", "104.18.6.192"] {
            assert!(!blocked_ip(ip(raw)), "{raw}");
        }
    }

    #[test]
    fn names_are_compared_without_case_and_trailing_dot() {
        assert!(blocked_name("Supervisor"));
        assert!(blocked_name("metadata.google.internal."));
        assert!(!blocked_name("ollama.local"));
    }

    #[test]
    fn one_blocked_address_refuses_the_whole_name() {
        let inward = vec!["93.184.216.34:0".parse().unwrap(), "169.254.169.254:0".parse().unwrap()];
        assert_eq!(admit("llm.example.test", inward).unwrap_err().kind(), io::ErrorKind::PermissionDenied);
        let outward: Vec<SocketAddr> = vec!["93.184.216.34:0".parse().unwrap()];
        assert_eq!(admit("llm.example.test", outward.clone()).unwrap(), outward);
    }
}
