//! SSL client
//!
//! This module implements a Rust version of the watchtowrConn.py script,
//! providing SSL/TLS connectivity.

use std::{
    fs::File,
    io::{self, BufReader, ErrorKind},
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};

use libafl::executors::ExitKind;
use log::debug;
use rustls::{
    ClientConfig, RootCertStore,
    pki_types::{CertificateDer, PrivateKeyDer},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::{sleep, timeout},
};
use tokio_rustls::TlsConnector;

/// SSL client
#[derive(Debug)]
pub struct RustlsClient {
    tls_config: Arc<ClientConfig>,
    server_addr: SocketAddr,
}

impl RustlsClient {
    /// Create a new RustlsClient with certificate and key files
    ///
    /// # Arguments
    /// * `cert_path` - Path to the client certificate file (e.g., "cert.bin")
    /// * `key_path` - Path to the client private key file (e.g., "key.bin")
    /// * `server_addr` - Server address (e.g., "192.168.1.111:514")
    pub fn new(cert_path: &str, key_path: &str, server_addr: SocketAddr) -> io::Result<Self> {
        // Load client certificate
        let cert_file = File::open(cert_path)?;
        let mut cert_reader = BufReader::new(cert_file);
        let certs = load_certs(&mut cert_reader)?;

        // Load client private key
        let key_file = File::open(key_path)?;
        let mut key_reader = BufReader::new(key_file);
        let key = load_private_key(&mut key_reader)?;

        // Create TLS config with client authentication
        let mut root_store = RootCertStore::empty();
        // Accept self-signed certificates (equivalent to verify_mode = CERT_NONE)
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let config = ClientConfig::builder()
            .dangerous() // Required to disable certificate verification
            .with_custom_certificate_verifier(Arc::new(NoVerifier))
            .with_client_auth_cert(certs, key)
            .map_err(|e| io::Error::new(ErrorKind::InvalidInput, e))?;

        Ok(Self {
            tls_config: Arc::new(config),
            server_addr,
        })
    }

    /// Connect to device with SSL/TLS
    pub async fn connect(&self) -> io::Result<RustlsConnection> {
        // Create TCP connection
        let tcp_stream = TcpStream::connect(&self.server_addr).await?;
        let ip_addr = self.server_addr.ip();
        let server_name = rustls::pki_types::ServerName::IpAddress(ip_addr.into());

        // Wrap with TLS
        let connector = TlsConnector::from(self.tls_config.clone());
        let server_name = rustls::pki_types::ServerName::try_from(server_name)
            .map_err(|e| io::Error::new(ErrorKind::InvalidInput, e))?
            .to_owned();

        let tls_stream = connector.connect(server_name, tcp_stream).await?;

        Ok(RustlsConnection { stream: tls_stream })
    }

    /// Execute input by Rustls client
    pub async fn execute_input(
        &self,
        input: &[u8],
    ) -> Result<(ExitKind, Vec<u8>), Box<dyn std::error::Error>> {
        let mut conn = {
            let mut retry = 0;
            loop {
                match self.connect().await {
                    Ok(c) => break c,
                    Err(e) => {
                        if retry >= 3 {
                            return Err(e.into());
                        }
                        retry += 1;
                        sleep(Duration::from_millis(50)).await;
                    }
                }
            }
        };

        if let Err(e) = conn.send_packet(&input).await {
            debug!("Socket error: {}", e);
            return Ok((ExitKind::Ok, vec![]));
        }

        let resp = match timeout(Duration::from_millis(500), conn.recv(1000)).await {
            Ok(Ok(response)) => response,
            Ok(Err(e)) => match e.kind() {
                ErrorKind::UnexpectedEof | ErrorKind::ConnectionReset | ErrorKind::BrokenPipe => {
                    return Ok((ExitKind::Ok, vec![]));
                }
                _ => {
                    println!("Socket error: {}", e);
                    return Err(e.into());
                }
            },
            Err(_) => {
                return Ok((ExitKind::Timeout, vec![]));
            }
        };

        let _ = conn.close().await;
        Ok((ExitKind::Ok, resp))
    }
}

/// Active SSL connection
#[derive(Debug)]
pub struct RustlsConnection {
    stream: tokio_rustls::client::TlsStream<TcpStream>,
}

impl RustlsConnection {
    /// Send a packet to the server
    pub async fn send_packet(&mut self, packet: &[u8]) -> io::Result<()> {
        self.stream.write_all(packet).await?;
        self.stream.flush().await?;
        Ok(())
    }

    /// Send raw bytes to the server
    pub async fn send_raw(&mut self, data: &[u8]) -> io::Result<()> {
        self.stream.write_all(data).await?;
        self.stream.flush().await?;
        Ok(())
    }

    /// Receive response from the server
    pub async fn recv(&mut self, max_len: usize) -> io::Result<Vec<u8>> {
        let mut buffer = vec![0u8; max_len];
        let n = self.stream.read(&mut buffer).await?;
        buffer.truncate(n);
        Ok(buffer)
    }

    /// Close the connection
    pub async fn close(mut self) -> io::Result<()> {
        self.stream.shutdown().await
    }
}

/// Load certificates from a file (PEM or DER format)
fn load_certs(reader: &mut dyn io::BufRead) -> io::Result<Vec<CertificateDer<'static>>> {
    // Try PEM first
    let mut pem_data = Vec::new();
    reader.read_to_end(&mut pem_data)?;

    // Try parsing as PEM
    if let Ok(certs) =
        rustls_pemfile::certs(&mut pem_data.as_slice()).collect::<Result<Vec<_>, _>>()
    {
        if !certs.is_empty() {
            return Ok(certs);
        }
    }

    // If not PEM, assume it's DER format
    Ok(vec![CertificateDer::from(pem_data)])
}

/// Load private key from a file (PEM or DER format)
fn load_private_key(reader: &mut dyn io::BufRead) -> io::Result<PrivateKeyDer<'static>> {
    let mut key_data = Vec::new();
    reader.read_to_end(&mut key_data)?;

    // Try parsing as PEM
    if let Ok(Some(key)) = rustls_pemfile::private_key(&mut key_data.as_slice()) {
        return Ok(key);
    }

    // If not PEM, assume it's DER format (PKCS#8)
    Ok(PrivateKeyDer::Pkcs8(key_data.into()))
}

/// Custom certificate verifier that accepts all certificates
/// (equivalent to Python's verify_mode = ssl.CERT_NONE)
#[derive(Debug)]
struct NoVerifier;

impl rustls::client::danger::ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
        ]
    }
}
