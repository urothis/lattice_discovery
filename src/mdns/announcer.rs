use crate::prelude::*;

use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::{collections::HashSet, fmt, ops::Deref};
use tokio::time::timeout;
use tracing::debug;

pub mod prelude {
    pub use super::Announce;
}

pub struct Announce {
    pub announcer: String,
    pub name: String,
    pub identifier: String,
    pub address: String,
    pub port: u16,
    pub udp: bool,
    pub https: bool,
    pub mdns_daemon: Option<ServiceDaemon>,
}

impl Announce {
    pub fn new(
        announcer: &str,
        name: &str,
        identifier: &str,
        port: u16,
        udp: bool,
        https: bool,
    ) -> Self {
        Self {
            announcer: announcer.to_string(),
            name: name.to_string(),
            identifier: identifier.to_string(),
            address: String::new(),
            port,
            udp,
            https,
            mdns_daemon: None,
        }
    }

    pub async fn search(find: Find) -> Result<Vec<Self>> {
        let mdns = ServiceDaemon::new()?;
        let receiver = mdns.browse(&format!(
            "_{}._{}.local.",
            find.name,
            if find.udp { "udp" } else { "tcp" }
        ))?;
        let mut seen_instances = HashSet::new();
        let mut instances = Vec::new();
        let mut search_result: Result<()> = Ok(());

        // we want to run for a bounded time
        // if we keep finding new services we keep going
        // until idle_timeout passes without new services
        loop {
            let event = match timeout(find.idle_timeout, receiver.recv_async()).await {
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
                let role = match props.get("role") {
                    Some(role) => {
                        let value = role.val_str().to_owned();
                        if value != find.role_to_find {
                            continue;
                        }
                        value
                    }
                    None => continue,
                };
                let uses_https = props
                    .get("transport")
                    .map(|transport| transport.val_str().eq_ignore_ascii_case("https"))
                    .unwrap_or(find.https);
                for addr in info.get_addresses() {
                    let scheme = if uses_https { "https" } else { "http" };
                    let address =
                        format!("{}://{}:{}/", scheme, addr.to_ip_addr(), info.get_port());
                    instances.push(Self {
                        announcer: role.clone(),
                        name: find.name.clone(),
                        identifier: info.get_fullname().to_string(),
                        address,
                        port: info.get_port(),
                        udp: find.udp,
                        https: uses_https,
                        mdns_daemon: None,
                    });
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

    pub async fn announce(&mut self, role: &str) -> Result<()> {
        let mdns = ServiceDaemon::new()?;
        debug!("Starting mDNS broadcaster");
        let service_type = format!(
            "_{}._{}.local.",
            self.name,
            if self.udp { "udp" } else { "tcp" }
        );
        let host_name = format!("{}.local.", self.name);
        let ips = collect_local_ips()?;
        let transport = if self.https { "https" } else { "http" };
        let properties = [("role", role), ("transport", transport)];
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

impl Deref for Announce {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.address
    }
}

impl fmt::Display for Announce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.address)
    }
}
