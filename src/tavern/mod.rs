//! Tavern has many kind of harnesses for different targets.

#[cfg(feature = "openssl")]
pub mod openssl;
pub mod option;
pub mod tcp_udp;
