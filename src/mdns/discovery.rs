use crate::prelude::*;

use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::{collections::HashSet, fmt, ops::Deref};
use tokio::time::timeout;
use tracing::debug;

pub mod prelude {
    pub use super::Discover;
}

pub struct Discover {
    pub role: String,
    pub name: String,
    pub identifier: String,
    pub address: String,
    pub port: u16,
    pub udp: bool,
    pub https: bool,
    pub mdns_daemon: Option<ServiceDaemon>,
}

impl Discover {
    pub fn new(
        role: &str,
        name: &str,
        identifier: &str,
        port: u16,
        udp: bool,
        https: bool,
    ) -> Self {
        Self {
            role: role.to_string(),
            name: name.to_string(),
            identifier: identifier.to_string(),
            address: String::new(),
            port,
            udp,
            https,
            mdns_daemon: None,
        }
    }

    pub async fn search(browse: Browse) -> Result<Vec<Self>> {
        let mdns = ServiceDaemon::new()?;

        // Determine which service types to browse. If no name is provided, discover
        // all advertised service types via the DNS-SD meta-service.
        let mut service_types = Vec::new();
        if let Some(name) = &browse.name {
            service_types.push(format!(
                "_{}._{}.local.",
                name,
                if browse.udp { "udp" } else { "tcp" }
            ));
        } else {
            let service_receiver = mdns.browse("_services._dns-sd._udp.local.")?;
            loop {
                let event = match timeout(browse.idle_timeout, service_receiver.recv_async()).await
                {
                    Ok(event) => event,
                    Err(_) => break,
                };
                let event = match event {
                    Ok(event) => event,
                    Err(_) => break,
                };
                if let ServiceEvent::ServiceResolved(info) = event {
                    service_types.push(info.get_fullname().to_string());
                }
            }
            // De-duplicate discovered service types.
            service_types.sort();
            service_types.dedup();
        }

        let mut seen_instances = HashSet::new();
        let mut instances = Vec::new();
        let mut search_result: Result<()> = Ok(());

        for service_type in service_types {
            let receiver = mdns.browse(&service_type)?;

            // we want to run for a bounded time
            // if we keep finding new services we keep going
            // until idle_timeout passes without new services
            loop {
                let event = match timeout(browse.idle_timeout, receiver.recv_async()).await {
                    Ok(event) => event,
                    Err(_) => break,
                };
                let event = match event {
                    Ok(event) => event,
                    Err(_) => {
                        search_result = Err(Error::MdnsChannelClosed);
                        break;
                    }
                };
                if let ServiceEvent::ServiceResolved(info) = event {
                    let instance = info.get_fullname().to_string();
                    debug!("Service resolved: {}", instance);
                    if !seen_instances.insert(instance.clone()) {
                        continue;
                    }
                    let props = info.get_properties();
                    let role = props.get("role").map(|role| role.val_str().to_owned());
                    if let Some(expected_role) = &browse.role
                        && role.as_deref() != Some(expected_role.as_str())
                    {
                        continue;
                    }
                    let uses_https = props
                        .get("transport")
                        .map(|transport| transport.val_str().eq_ignore_ascii_case("https"))
                        .unwrap_or(browse.https);
                    for addr in info.get_addresses() {
                        let scheme = if uses_https { "https" } else { "http" };
                        let address =
                            format!("{}://{}:{}/", scheme, addr.to_ip_addr(), info.get_port());
                        let name = browse
                            .name
                            .clone()
                            .unwrap_or_else(|| service_type.trim_end_matches('.').to_string());
                        instances.push(Self {
                            role: role.clone().unwrap_or_default(),
                            name,
                            identifier: info.get_fullname().to_string(),
                            address,
                            port: info.get_port(),
                            udp: browse.udp,
                            https: uses_https,
                            mdns_daemon: None,
                        });
                    }
                }
            }
        }
        let shutdown_result = mdns.shutdown().map(|_| ());

        match (search_result, shutdown_result) {
            (Ok(()), Ok(())) => Ok(instances),
            (Err(err), Ok(())) => Err(err),
            (Ok(()), Err(shutdown_err)) => Err(shutdown_err.into()),
            (Err(err), Err(shutdown_err)) => {
                debug!("mDNS shutdown failed after search error: {}", shutdown_err);
                Err(err)
            }
        }
    }

    pub async fn publish(&mut self) -> Result<()> {
        let mdns = ServiceDaemon::new()?;
        debug!("Publishing service over mDNS");
        let service_type = format!(
            "_{}._{}.local.",
            self.name,
            if self.udp { "udp" } else { "tcp" }
        );
        let host_name = format!("{}.local.", self.name);
        let ips = collect_local_ips()?;
        let transport = if self.https { "https" } else { "http" };
        let properties = [("role", self.role.as_str()), ("transport", transport)];
        let service_info = ServiceInfo::new(
            &service_type,
            &self.identifier,
            &host_name,
            &ips[..],
            self.port,
            &properties[..],
        )?;
        debug!("Service info: {:?}", service_info);
        mdns.register(service_info)?;
        self.mdns_daemon = Some(mdns);
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(mdns) = &self.mdns_daemon {
            debug!("Stopping mDNS broadcaster");
            mdns.shutdown()?;
            self.mdns_daemon = None;
        }
        Ok(())
    }
}

impl Deref for Discover {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.address
    }
}

impl fmt::Display for Discover {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.address)
    }
}
