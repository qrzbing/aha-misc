//! OpenSSL crates need some package installation,
//! please read https://docs.rs/openssl/latest/openssl/

use std::{io::Write, net::TcpStream, path::Path, sync::Arc, time::Duration};

use libafl::{executors::ExitKind, inputs::BytesInput};
use log::debug;
use openssl::ssl::{SslConnector, SslFiletype, SslMethod, SslOptions, SslStream, SslVerifyMode};

use crate::tavern::option;

/// Create an insecure SSL stream
pub fn create_insecure_ssl_stream(
    host: &str,
    port: u16,
    client_cert: Option<&Path>,
    client_key: Option<&Path>,
) -> Result<SslStream<TcpStream>, Box<dyn std::error::Error>> {
    let mut builder = SslConnector::builder(SslMethod::tls())?;

    // Disable certificate verification
    builder.set_verify(SslVerifyMode::NONE);
    builder.set_options(SslOptions::ALLOW_UNSAFE_LEGACY_RENEGOTIATION);

    if let (Some(cert_path), Some(key_path)) = (client_cert, client_key) {
        builder.set_certificate_file(cert_path, SslFiletype::PEM)?;
        builder.set_private_key_file(key_path, SslFiletype::PEM)?;

        // Check if the private key matches the certificate
        if let Err(e) = builder.check_private_key() {
            return Err(Box::new(e));
        }
    }

    let connector = builder.build();

    let stream = TcpStream::connect((host, port))?;

    let ssl_stream = connector
        .configure()?
        .verify_hostname(false)
        .connect(host, stream)?;

    Ok(ssl_stream)
}

/// An openssl harness
pub fn create_openssl_harness(
    network: option::NetworkClientOptions,
    ssl: option::SSLOptions,
) -> impl Fn(&BytesInput) -> ExitKind {
    let connector = {
        let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();

        builder.set_verify(SslVerifyMode::NONE);

        builder.set_options(SslOptions::ALLOW_UNSAFE_LEGACY_RENEGOTIATION);

        if let (Some(cert_path), Some(key_path)) = (ssl.cert_file, ssl.key_file) {
            debug!("Loading client certificate: {:?}", cert_path);
            debug!("Loading client private key: {:?}", key_path);

            builder
                .set_certificate_file(&cert_path, SslFiletype::PEM)
                .expect("Failed to load client certificate file");
            builder
                .set_private_key_file(&key_path, SslFiletype::PEM)
                .expect("Failed to load client private key file");
            builder
                .check_private_key()
                .expect("Client private key is invalid");
        } else {
            debug!("No client certificate and key provided, continuing in client-only mode.");
        }

        Arc::new(builder.build())
    };

    let host_for_closure = network.host;

    move |input: &BytesInput| {
        let tcp_stream = match TcpStream::connect((host_for_closure.as_str(), network.port)) {
            Ok(stream) => stream,
            Err(e) => {
                panic!(
                    "Cannot connect to {}:{}: {}",
                    host_for_closure, network.port, e
                );
            }
        };

        let mut ssl_stream = match connector
            .configure()
            .unwrap()
            .verify_hostname(false)
            .connect(&host_for_closure, tcp_stream)
        {
            Ok(stream) => stream,
            Err(e) => {
                panic!("TLS handshake failed: {}", e);
            }
        };

        if let Err(e) = ssl_stream.write_all(input.as_ref()) {
            debug!("Send error:: {}", e);
        }

        std::thread::sleep(Duration::from_nanos(network.sleep));

        ExitKind::Ok
    }
}
