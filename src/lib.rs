//! Discover crate for service discovery and advertisement using mDNS
//!
//! Example usage of the discover crate
//!
//! ```rust,no_run
//! use discover::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Register a service for discovery
//!     let mut discovery = Discover::new(
//!         // publishing role/name
//!         "server",
//!         // service name
//!         "my_service",
//!         // unique identifier
//!         "instance1",
//!         // port to publish for discovery
//!         8080,
//!         // use UDP
//!         false,
//!         // use HTTPS when advertising
//!         true,
//!     );
//!     discovery.publish().await?;
//!
//!     // Discover services of a specific role
//!     // returns: Vec<Discover> describing discovered services (one per address)
//!     // [
//!     //   Discover { address: "http://found_service_1:8080/", ... },
//!     //   Discover { address: "http://found_service_2:8080/", ... },
//!     // ]
//!     let discovered_services = Discover::search(Browse::new(
//!         // Service name
//!         Some("my_service"), // None to browse all services
//!         // role to find
//!         Some("server"),
//!         // use UDP
//!         false,
//!         // use HTTPS fallback for discovered services without advertised transport
//!         false,
//!         // idle timeout for discovery (stop when nothing arrives for this long)
//!         std::time::Duration::from_secs(1),
//!     ))
//!     .await?;
//!
//!     // Stop broadcasting discovery information
//!     discovery.stop().await?;
//!
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod mdns;
pub mod utility;

pub mod prelude {
    pub use super::{error::*, mdns::prelude::*, utility::prelude::*};
}
