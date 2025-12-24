//! Some options

use std::path::PathBuf;

use clap::Args;

/// Network protocol types supported by the client.
#[derive(Debug, Clone, Copy)]
pub enum Protocol {
    /// TCP protocol
    Tcp,
    /// UDP protocol
    Udp,
}

/// Options for configuring network client fuzzing.
///
/// This struct defines parameters needed for fuzzing network clients,
/// including protocol type, host, port, and timing configurations.
#[derive(Args, Debug)]
pub struct NetworkClientOptions {
    /// Network protocol (tcp/udp)
    #[arg(long, default_value = "tcp")]
    pub protocol: String,

    /// Target host
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// Target port
    #[arg(long, default_value = "13337")]
    pub port: u16,

    /// Sleep time in microseconds
    #[arg(long, default_value = "100000")]
    pub sleep: u64,

    /// Restart try time if connection fails
    #[arg(long, default_value = "10")]
    pub try_time: u64,
}

impl NetworkClientOptions {
    /// Get the protocol enum from the string configuration.
    ///
    /// # Returns
    /// A `Protocol` enum value corresponding to the configured protocol string.
    ///
    /// # Panics
    /// Panics if an invalid protocol is specified.
    pub fn protocol(&self) -> Protocol {
        match self.protocol.as_str() {
            "tcp" => Protocol::Tcp,
            "udp" => Protocol::Udp,
            _ => panic!("Invalid protocol specified"),
        }
    }
}

/// SSL configuration options.
///
/// This struct defines the parameters needed for SSL connections,
/// including certificate and key file paths.
#[derive(Args, Debug)]
pub struct SSLOptions {
    /// SSL cert file
    #[arg(long = "cert-file")]
    pub cert_file: Option<PathBuf>,

    /// SSL key file
    #[arg(long = "key-file")]
    pub key_file: Option<PathBuf>,
}
