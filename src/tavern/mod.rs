//! Tavern has many kind of harnesses for different targets.

#[cfg(feature = "openssl")]
pub mod openssl;
pub mod option;
#[cfg(feature = "rustls")]
pub mod rustls;
pub mod tcp_udp;
