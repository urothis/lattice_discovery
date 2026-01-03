# Announce

Utility crate for advertising and discovering services on the local network over mDNS.

## Features
- Broadcasts HTTP/HTTPS or TCP/UDP services with a role tag and advertised transport.
- Discovers peers by name and role, returning base URLs using the announcer-provided transport when present.
- Uses a short idle timeout so discovery returns promptly when nothing is present.

## Usage

### Announcing a service
```rust
use announce::prelude::*;

let mut announcer = Announce::new(
    "my_role",    // announcer role
    "my_service", // service name
    "instance-1", // unique identifier
    8080,         // port
    false,        // udp?
    true,         // https?
);

// Broadcast over mDNS
announcer.announce("agent").await?;

// Later, shut down the broadcaster
announcer.stop().await?;
```

### Discovering peers
`Announce::search` browses mDNS for services that match a name/role and returns their `Announce` metadata, one entry per discovered base URL (accessible via `address`). It stops if nothing is found for one second.
```rust
use announce::prelude::*;

let peers = Announce::search(Find::new(
    "my_service",                      // service name
    "agent",                           // role to match
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
- Local interfaces are collected via `if_addrs`; announcement fails (with a typed error) if interface enumeration fails or no non-loopback addresses exist.
- Discovery results are not HTTP-validated; perform any health checks yourself after discovery if needed.
- URLs in the discovery result include a trailing `/` (e.g. `http://192.168.1.10:8080/`).
- The final `Duration` in `Find::new` controls how long discovery waits for mDNS responses after the last event.
- Discovery prefers the announcer’s advertised `transport` property (`http` or `https`), falling back to the `https` flag on `Find` only when the property is missing.

## Errors
- `Mdns`: Underlying `mdns_sd` error when creating, browsing, registering, or shutting down.
- `MdnsChannelClosed`: The browse channel closed unexpectedly during discovery.
- `InterfaceEnumerationFailed(std::io::Error)`: Failed to enumerate network interfaces via `if_addrs`.
- `NoUsableInterfaces`: No non-loopback, non-link-local IP addresses were found to announce on.
