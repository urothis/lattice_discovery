use mdns_sd::Error as MdnsError;
use std::{error, fmt};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Mdns(MdnsError),
    MdnsChannelClosed,
    InterfaceEnumerationFailed(std::io::Error),
    NoUsableInterfaces,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Mdns(err) => write!(f, "mDNS error: {}", err),
            Error::MdnsChannelClosed => write!(f, "mDNS event channel closed"),
            Error::InterfaceEnumerationFailed(err) => {
                write!(f, "Failed to enumerate network interfaces: {err}")
            }
            Error::NoUsableInterfaces => write!(f, "No usable network interfaces found"),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Mdns(err) => Some(err),
            Error::MdnsChannelClosed => None,
            Error::InterfaceEnumerationFailed(err) => Some(err),
            Error::NoUsableInterfaces => None,
        }
    }
}

impl From<MdnsError> for Error {
    fn from(err: MdnsError) -> Self {
        Error::Mdns(err)
    }
}
