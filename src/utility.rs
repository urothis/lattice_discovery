use crate::error::{Error, Result};
use if_addrs::get_if_addrs;
use std::net::IpAddr;
use tracing::debug;

pub mod prelude {
    pub use super::collect_local_ips;
}

/// Collect all non-loopback, non-link-local IP addresses on the local machine
pub fn collect_local_ips() -> Result<Vec<IpAddr>> {
    let ips: Vec<_> = get_if_addrs()
        .map_err(Error::InterfaceEnumerationFailed)?
        .into_iter()
        .filter_map(|iface| {
            if iface.is_loopback() {
                return None;
            }
            debug!("Found interface: {} with IP {}", iface.name, iface.ip());
            match iface.ip() {
                IpAddr::V4(ip) if ip.octets()[0] != 169 => Some(IpAddr::V4(ip)),
                IpAddr::V6(ip) if !ip.is_unicast_link_local() => Some(IpAddr::V6(ip)),
                _ => None,
            }
        })
        .collect();

    if ips.is_empty() {
        return Err(Error::NoUsableInterfaces);
    }

    Ok(ips)
}
