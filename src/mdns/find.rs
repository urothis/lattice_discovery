pub mod prelude {
    pub use super::Find;
}

#[derive(Clone, Debug)]
pub struct Find {
    pub name: String,
    pub role_to_find: String,
    pub udp: bool,
    pub https: bool,
    pub idle_timeout: std::time::Duration,
}

impl Find {
    pub fn new(
        name: &str,
        role_to_find: &str,
        udp: bool,
        https: bool,
        idle_timeout: std::time::Duration,
    ) -> Self {
        Self {
            name: name.to_string(),
            role_to_find: role_to_find.to_string(),
            udp,
            https,
            idle_timeout,
        }
    }
}
