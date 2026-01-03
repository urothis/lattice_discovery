pub mod prelude {
    pub use super::Browse;
}

#[derive(Clone, Debug)]
pub struct Browse {
    pub name: Option<String>,
    pub role: Option<String>,
    pub udp: bool,
    pub https: bool,
    pub idle_timeout: std::time::Duration,
}

impl Browse {
    pub fn new(
        name: Option<&str>,
        role: Option<&str>,
        udp: bool,
        https: bool,
        idle_timeout: std::time::Duration,
    ) -> Self {
        Self {
            name: name.map(|n| n.to_string()),
            role: role.map(|r| r.to_string()),
            udp,
            https,
            idle_timeout,
        }
    }
}
