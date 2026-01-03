# Discover

Utility crate for advertising and discovering services on the local network over mDNS.

## Features
- Broadcasts HTTP/HTTPS or TCP/UDP services with a role tag and advertised transport.
- Discovers peers by name and role, returning base URLs using the discovery-provided transport when present.
- Uses a short idle timeout so discovery returns promptly when nothing is present.

## Usage

### Registering a service for discovery
```rust
use discover::prelude::*;

let mut discovery = Discover::new(
    "my_role",    // publishing role
    "my_service", // service name
    "instance-1", // unique identifier
    8080,         // port
    false,        // udp?
    true,         // https?
);

// Broadcast over mDNS
discovery.publish().await?;

// Later, shut down the broadcaster
discovery.stop().await?;
```

### Discovering peers
`Discover::search` browses mDNS for services that match a name/role and returns their `Discover` metadata, one entry per discovered base URL (accessible via `address`). It stops if nothing is found for one second.
```rust
use discover::prelude::*;

let peers = Discover::search(Browse::new(
    None,                              // service name (None = all mDNS services)
    Some("agent"),                     // role to match (None to ignore)
    false,                             // udp?
    false,                             // https? (used as a fallback when transport isn't advertised)
    std::time::Duration::from_secs(1), // idle timeout
))
.await?;

for peer in peers {
    println!("Found peer at {}", peer.address);
}
```

## Behavior notes
- Local interfaces are collected via `if_addrs`; publishing fails (with a typed error) if interface enumeration fails or no non-loopback addresses exist.
- Discovery results are not HTTP-validated; perform any health checks yourself after discovery if needed.
- URLs in the discovery result include a trailing `/` (e.g. `http://192.168.1.10:8080/`).
- The final `Duration` in `Browse::new` controls how long discovery waits for mDNS responses after the last event.
- Discovery prefers the publisher’s advertised `transport` property (`http` or `https`), falling back to the `https` flag on `Browse` only when the property is missing.
- Passing `None` for the service name will first discover all advertised mDNS service types, then browse each one (with the same idle timeout).

## Errors
- `Mdns`: Underlying `mdns_sd` error when creating, browsing, registering, or shutting down.
- `MdnsChannelClosed`: The browse channel closed unexpectedly during discovery.
- `InterfaceEnumerationFailed(std::io::Error)`: Failed to enumerate network interfaces via `if_addrs`.
- `NoUsableInterfaces`: No non-loopback, non-link-local IP addresses were found to publish on.
