use discover::prelude::*;
use std::io::Read;

#[tokio::main]
async fn main() -> Result<()> {
    // Publish a demo service for others to browse.
    let mut discovery = Discover::new(
        "server",                                // role tag other peers match on
        "demo-service",                          // service name (must match on clients)
        &format!("demo-{}", std::process::id()), // instance identifier
        8080,                                    // service port
        false,                                   // udp?
        false,                                   // https?
    );

    discovery.publish().await?;
    println!("Published demo-service on port 8080 as role 'server'. Press Enter to stop.");

    // Keep running until user presses Enter (avoids extra Tokio signal features).
    let _ = std::io::stdin().read(&mut [0u8]).ok();

    discovery.stop().await?;
    println!("Stopped publishing.");
    Ok(())
}
