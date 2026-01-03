//! Announce crate for service discovery and announcement using mDNS
//!
//! Example usage of the announce crate
//!
//! ```rust,no_run
//! use announce::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Announce a service
//!     let mut announcer = Announce::new(
//!         // announcer role/name
//!         "server",
//!         // service name
//!         "my_service",
//!         // unique identifier
//!         "instance1",
//!         // port to announce
//!         8080,
//!         // use UDP
//!         false,
//!         // use HTTPS
//!         true,
//!     );
//!     announcer.announce("server").await?;
//!
//!     // Discover services of a specific role
//!     // returns: Vec<Announce> describing discovered services (one per address)
//!     // [
//!     //   Announce { address: "http://found_service_1:8080/", ... },
//!     //   Announce { address: "http://found_service_2:8080/", ... },
//!     // ]
//!     let discovered_services = Announce::search(Find::new(
//!         // Service name
//!         "my_service",
//!         // role to find
//!         "server",
//!         // use UDP
//!         false,
//!         // use HTTPS
//!         false, // used as a fallback if the announcer does not advertise transport
//!         // idle timeout for discovery (stop when nothing arrives for this long)
//!         std::time::Duration::from_secs(1),
//!     ))
//!     .await?;
//!
//!     // Stop announcing
//!     announcer.stop().await?;
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
