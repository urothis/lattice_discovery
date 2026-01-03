use discover::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Browse for demo-service servers published on the local network.
    let peers = Discover::search(Browse::new(
        Some("demo-service"),   // service name (None to browse all services)
        Some("server"),         // role to match (None to match any role)
        false,                  // udp?
        false,                  // https? fallback if transport not advertised
        Duration::from_secs(2), // idle timeout while browsing
    ))
    .await?;

    if peers.is_empty() {
        println!("No demo-service peers found.");
    } else {
        println!("Found peers:");
        for peer in peers {
            println!("- {} at {}", peer.role, peer.address);
        }
    }

    Ok(())
}
