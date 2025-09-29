//! TCP && UDP harness

use std::{
    io::{self, Read, Write},
    net::{TcpStream, ToSocketAddrs, UdpSocket},
    os::unix::io::AsRawFd,
    thread::sleep,
    time::Duration,
};

use libafl::{HasMetadata, executors::ExitKind, inputs::HasTargetBytes};
use libafl_bolts::AsSlice;
use libc::{self, socklen_t};
use log::error;

use crate::tavern::option::{NetworkClientOptions, Protocol};

/// Internal connection type representation.
#[derive(Debug)]
enum Connection {
    /// TCP stream connection
    Tcp(TcpStream),
    /// UDP socket connection
    Udp(UdpSocket),
}

/// A client for network-based fuzzing that supports both TCP and UDP protocols.
#[derive(Debug)]
pub struct NetworkClient {
    /// The protocol to use (TCP or UDP)
    protocol: Protocol,
    /// Timeout for connection and I/O operations
    timeout: Duration,
    /// Current active connection
    connection: Option<Connection>,
    /// Target server address
    server_addr: Option<String>,
}

impl NetworkClient {
    /// Create a new NetworkClient with the specified protocol and timeout.
    ///
    /// # Arguments
    /// * `protocol` - The network protocol to use (TCP or UDP)
    /// * `timeout_usecs` - Connection and I/O timeout in microseconds
    ///
    /// # Returns
    /// A new NetworkClient instance
    pub fn new(protocol: Protocol, timeout_usecs: u64) -> Self {
        Self {
            protocol,
            timeout: Duration::from_micros(timeout_usecs),
            connection: None,
            server_addr: None,
        }
    }

    /// Configure the socket's SO_LINGER option.
    ///
    /// # Arguments
    /// * `fd` - File descriptor of the socket to configure
    ///
    /// # Returns
    /// `Ok(())` on success, or an error if the operation fails
    ///
    /// # Description
    /// Sets the SO_LINGER option with a timeout of 0 seconds,
    /// which causes the socket to close immediately on shutdown.
    fn set_linger(&self, fd: i32) -> io::Result<()> {
        let linger = libc::linger {
            l_onoff: 1,  // enable SO_LINGER
            l_linger: 0, // set timeout to 0
        };

        let result = unsafe {
            libc::setsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_LINGER,
                &linger as *const _ as *const libc::c_void,
                size_of::<libc::linger>() as socklen_t,
            )
        };

        if result < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    /// Connect to a target server.
    ///
    /// # Arguments
    /// * `host` - Hostname or IP address of the target server
    /// * `port` - Port number of the target server
    ///
    /// # Returns
    /// `Ok(())` on successful connection, or an error if connection fails
    ///
    /// # Description
    /// For TCP, this performs the actual connection and will retry up to 1000 times.
    /// For UDP, this sets up the socket but doesn't establish a connection.
    pub fn connect_to_server(&mut self, host: &str, port: u16) -> io::Result<()> {
        let addr = format!("{}:{}", host, port);
        self.server_addr = Some(addr.clone());

        match self.protocol {
            Protocol::Tcp => {
                let addr = addr.to_socket_addrs()?.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::Other, "Failed to resolve address")
                })?;

                let mut last_error = None;
                for _ in 0..1000 {
                    match TcpStream::connect_timeout(&addr, self.timeout) {
                        Ok(stream) => {
                            // Set SO_LINGER
                            self.set_linger(stream.as_raw_fd())?;

                            stream.set_read_timeout(Some(self.timeout))?;
                            stream.set_write_timeout(Some(self.timeout))?;
                            self.connection = Some(Connection::Tcp(stream));
                            return Ok(());
                        }
                        Err(e) => {
                            last_error = Some(e);
                            println!(
                                "Failed to connect to {}: {}",
                                addr,
                                last_error.as_ref().unwrap()
                            );
                            sleep(Duration::from_micros(1000));
                        }
                    }
                }

                Err(last_error.unwrap_or_else(|| {
                    io::Error::new(io::ErrorKind::Other, "Failed to connect after retries")
                }))
            }
            Protocol::Udp => {
                let socket = UdpSocket::bind("0.0.0.0:0")?;

                // Set SO_LINGER
                self.set_linger(socket.as_raw_fd())?;

                socket.set_read_timeout(Some(self.timeout))?;
                socket.set_write_timeout(Some(self.timeout))?;
                self.connection = Some(Connection::Udp(socket));
                Ok(())
            }
        }
    }

    /// Send a message to the connected server.
    ///
    /// # Arguments
    /// * `message` - The byte array to send
    ///
    /// # Returns
    /// `Ok(())` on successful send, or an error if sending fails
    ///
    /// # Description
    /// For TCP, this writes directly to the stream.
    /// For UDP, this sends a datagram to the previously specified server address.
    pub fn send_message(&mut self, message: &[u8]) -> io::Result<()> {
        match (&mut self.connection, self.server_addr.as_ref()) {
            (Some(Connection::Tcp(stream)), _) => {
                stream.write_all(message)?;
            }
            (Some(Connection::Udp(socket)), Some(addr)) => {
                socket.send_to(message, addr)?;
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::NotConnected,
                    "No active connection",
                ));
            }
        }
        Ok(())
    }

    /// Receive a response from the server.
    ///
    /// # Returns
    /// `Ok(Vec<u8>)` containing the received data on success, or an error if receiving fails
    ///
    /// # Description
    /// Reads up to 2048 bytes from the connection and returns the actual data received.
    #[allow(unused)]
    fn receive_response(&mut self) -> io::Result<Vec<u8>> {
        let mut buffer = vec![0; 2048];

        let n = match &mut self.connection {
            Some(Connection::Tcp(stream)) => stream.read(&mut buffer)?,
            Some(Connection::Udp(socket)) => {
                let (n, _) = socket.recv_from(&mut buffer)?;
                n
            }
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotConnected,
                    "No active connection",
                ));
            }
        };

        buffer.truncate(n);
        Ok(buffer)
    }
}

/// Create a default network fuzzing harness.
///
/// # Arguments
/// * `network` - Network client options containing connection details
///
/// # Returns
/// A closure that takes a `BytesInput` and returns an `ExitKind`.
///
/// # Description
/// The returned harness function will:
/// 1. Connect to the server specified in the network options
/// 2. Send the input data
/// 3. Wait for the specified delay time
/// 4. Return ExitKind::Ok
pub fn create_default_harness<S, I>(
    network: &NetworkClientOptions,
) -> impl FnMut(&mut S, &I) -> ExitKind
where
    S: HasMetadata,
    I: HasTargetBytes,
{
    let mut client = NetworkClient::new(network.protocol(), network.sleep);

    move |_state: &mut S, input: &I| {
        let mut do_send = false;
        for _ in 0..network.try_time {
            if let Err(_) = client.connect_to_server(&network.host, network.port) {
                error!("Failed to connect to server");
                sleep(Duration::from_secs(1));
                continue;
            }
            if let Err(_) = client.send_message(input.target_bytes().as_slice()) {
                error!("Failed to send message to server");
                sleep(Duration::from_secs(1));
                continue;
            }
            do_send = true;
            break;
        }
        if !do_send {
            panic!(
                "Failed to send message to server in {} tries",
                network.try_time
            )
        }
        sleep(Duration::from_nanos(network.sleep));

        ExitKind::Ok
    }
}
