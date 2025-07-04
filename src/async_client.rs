﻿use crate::shared::{ClientError, Credentials};
use std::borrow::Cow;
use std::pin::Pin;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_native_tls::TlsConnector;

trait Io: AsyncRead + AsyncWrite {}
impl<T: AsyncRead + AsyncWrite + ?Sized> Io for T {}
type DynStream = Pin<Box<dyn Io + Send>>;

/// A client for interacting with a RAC server.
///
/// The `Client` provides methods to connect to a RAC server, send and receive messages,
/// and manage user registration for `RACv2` connections.
///
/// # Example
///
/// ```no_run
/// use rac_rs::async_client::Client;
/// use rac_rs::shared::Credentials;
///
/// let credentials = Credentials {
///     username: "test_user".to_string(),
///     password: Some("password123".to_string()),
/// };
///
/// let mut client = Client::new(
///     "127.0.0.1:42666",
///     credentials,
///     false
/// );
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    /// The current size of messages in the client.
    current_messages_size: usize,
    /// The address of the RAC server.
    address: String,
    /// The username for authentication.
    username: String,
    /// The password for authentication, if required.
    password: Option<String>,
    /// Whether to use TLS encryption.
    use_tls: bool,
}

impl Client {
    /// Creates a new `Client` instance.
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the RAC server (e.g., "127.0.0.1:42666").
    /// * `credentials` - The username and optional password.
    /// * `connection` - The type of connection (`RAC` or `RACv2`).
    /// * `use_tls` - Whether to use TLS encryption for the connection.
    pub fn new(address: &str, credentials: Credentials, use_tls: bool) -> Self {
        Self {
            current_messages_size: 0,
            address: address.to_string(),
            username: credentials.username,
            password: credentials.password,
            use_tls,
        }
    }

    /// Updates the client's credentials.
    ///
    /// This method allows you to change the username and password for the client.
    pub fn update_credentials(&mut self, credentials: Credentials) {
        self.username = credentials.username;
        self.password = credentials.password;
    }

    /// Updates the client's TLS usage.
    ///
    /// This method allows you to enable or disable TLS encryption for the connection.
    pub fn update_tls(&mut self, use_tls: bool) {
        self.use_tls = use_tls;
    }

    /// Updates the client's address to the server.
    ///
    /// This method allows you to change the address of the RAC server.
    pub fn update_address(&mut self, address: String) {
        self.address = address;
    }

    /// Attempts to establish a TCP connection to the RAC server.
    async fn get_stream(&self) -> Result<DynStream, ClientError> {
        let stream = TcpStream::connect(&self.address)
            .await
            .map_err(ClientError::ConnectionError)?;

        if self.use_tls {
            let connector = TlsConnector::from(
                native_tls::TlsConnector::new()
                    .map_err(|e| ClientError::TlsInitializationError(e.to_string()))?,
            );

            let domain =
                self.address
                    .split(':')
                    .next()
                    .ok_or(ClientError::TlsInitializationError(
                        "Invalid address format".to_string(),
                    ))?;

            let tls_stream = connector
                .connect(domain, stream)
                .await
                .map_err(|e| ClientError::TlsInitializationError(e.to_string()))?;

            Ok(Box::pin(tls_stream))
        } else {
            Ok(Box::pin(stream))
        }
    }

    /// Removes null bytes from the data vector.
    /// 
    /// This is required because some servers that are written in C
    /// may send null bytes in the response, which can cause issues
    /// when parsing the response.
    fn remove_nulls(data: &mut Vec<u8>) {
        data.retain(|&x| x != 0);
    }

    /// Tests the connection to the RAC server.
    ///
    /// This method attempts to establish a TCP connection and returns `Ok(())` if successful.
    pub async fn test_connection(&self) -> Result<(), ClientError> {
        self.get_stream().await?;
        Ok(())
    }

    /// Registers a new user on the RAC server.
    ///
    /// # Errors
    ///
    /// Returns `ClientError::NoPassword` if no password specified for the client.
    /// Returns `ClientError::UsernameAlreadyTaken` if the username is already in use.
    /// Returns `ClientError::UnexpectedResponse` if got unexpected response from server.
    pub async fn register_user(&mut self) -> Result<(), ClientError> {
        // Getting the TCP stream to the RAC server.
        let mut stream = self.get_stream().await?;

        // Sending the username and password to the RAC server.
        if self.password.is_some() {
            stream
                .write_all(
                    format!(
                        "\x03{}\n{}",
                        self.username,
                        self.password.as_deref().unwrap()
                    )
                    .as_bytes(),
                )
                .await
                .map_err(ClientError::StreamWriteError)?;
            let mut buf = [0u8; 2];
            let n = stream
                .read(&mut buf)
                .await
                .map_err(ClientError::StreamReadError)?;
            if n == 0 {
                return Ok(());
            }
            match buf[0] {
                0x01 => Err(ClientError::UsernameAlreadyTaken),
                _ => Err(ClientError::UnexpectedResponse(
                    String::from_utf8_lossy(&buf[..n]).to_string(),
                )),
            }
        } else {
            Err(ClientError::NoPassword)
        }
    }

    /// Fetches the total size of all messages on the server and updates the client's internal state.
    ///
    /// This is useful for determining the amount of data to fetch for all messages.
    pub async fn fetch_messages_size(&mut self) -> Result<(), ClientError> {
        // Getting the TCP stream to the RAC server.
        let mut stream = self.get_stream().await?;

        // Trying to send 0x00 byte to get the size of messages.
        stream
            .write_all(&[0x00])
            .await
            .map_err(ClientError::StreamWriteError)?;

        let mut buf = vec![0u8; 1024];
        let n = stream
            .read(&mut buf)
            .await
            .map_err(ClientError::StreamReadError)?;

        if n == 0 {
            return Err(ClientError::ServerClosedConnection);
        }

        Self::remove_nulls(&mut buf);

        // Then, converting it to utf8 and parsing the size to usize.
        let response = String::from_utf8_lossy(&buf[..n]);
        if let Ok(size) = response.parse::<usize>() {
            self.current_messages_size = size;
            Ok(())
        } else {
            Err(ClientError::ParseError(
                "Failed to parse messages size".to_string(),
            ))
        }
    }

    /// Fetches all messages from the RAC server.
    ///
    /// This method retrieves all messages stored on the server and updates the
    /// client's internal message size tracker.
    pub async fn fetch_all_messages(&mut self) -> Result<Vec<Cow<str>>, ClientError> {
        let mut stream = self.get_stream().await?;

        // Sending 0x00 byte to get the size of messages.
        stream
            .write_all(&[0x00])
            .await
            .map_err(ClientError::StreamWriteError)?;
        let mut head = vec![0u8; 1024];
        let n = stream
            .read(&mut head)
            .await
            .map_err(ClientError::StreamReadError)?;
        
        if n == 0 {
            return Err(ClientError::ServerClosedConnection);
        }
        
        Self::remove_nulls(&mut head);
        let response = String::from_utf8_lossy(&head[..n]);
        let size = response
            .parse::<usize>()
            .map_err(|_| ClientError::ParseError("Failed to parse messages size".to_string()))?;
        self.current_messages_size = size;

        // Sending 0x01 byte to get all messages.
        stream
            .write_all(&[0x01])
            .await
            .map_err(ClientError::StreamWriteError)?;

        let mut buffer = vec![0u8; self.current_messages_size];
        stream
            .read_exact(&mut buffer)
            .await
            .map_err(ClientError::StreamReadError)?;

        Self::remove_nulls(&mut buffer);

        let response = String::from_utf8_lossy(&buffer).into_owned();

        let vec_messages = response
            .lines()
            .filter(|l| !l.is_empty())
            .map(|s| Cow::Owned(s.to_string()))
            .collect();

        Ok(vec_messages)
    }

    /// Fetches only new messages that have arrived since the last fetch.
    ///
    /// This method compares the current message size on the server with the client's
    /// stored size and retrieves only the difference. The client's internal message
    /// size tracker is updated upon successful fetch.
    pub async fn fetch_new_messages(&mut self) -> Result<Vec<Cow<str>>, ClientError> {
        // For this approach, we will not use fetch_messages_size function,
        // because it is necessary to fetch messages size AND THEN get new messages
        // IN THE SAME STREAM. Welcome to the Sugoma's bullshit protocol.

        let mut stream = self.get_stream().await?;

        // Sending 0x00 byte to get the size of messages.
        stream
            .write_all(&[0x00])
            .await
            .map_err(ClientError::StreamWriteError)?;
        let mut head = vec![0u8; 1024];
        let n = stream
            .read(&mut head)
            .await
            .map_err(ClientError::StreamReadError)?;

        if n == 0 {
            return Err(ClientError::ServerClosedConnection);
        }

        Self::remove_nulls(&mut head);

        // Then, converting it to utf8 and parsing the size to usize.
        let response = String::from_utf8_lossy(&head[..n]);
        let size = response
            .parse::<usize>()
            .map_err(|_| ClientError::ParseError("Failed to parse messages size".to_string()))?;

        // Now, we can get new messages.
        stream
            .write_all(format!("\x02{}", self.current_messages_size).as_bytes())
            .await
            .map_err(ClientError::StreamWriteError)?;

        let mut buffer = vec![0u8; size - self.current_messages_size];
        stream
            .read_exact(&mut buffer)
            .await
            .map_err(ClientError::StreamReadError)?;
        let response = String::from_utf8_lossy(&buffer).into_owned();

        Self::remove_nulls(&mut buffer);

        let vec_messages = response
            .lines()
            .filter(|l| !l.is_empty())
            .map(|s| Cow::Owned(s.to_string()))
            .collect();

        // Setting the new messages size.
        self.current_messages_size = size;

        Ok(vec_messages)
    }

    /// Sends a message to the server.
    ///
    /// The placeholder `{username}` in the message will be replaced with the client's username.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use rac_rs::async_client::Client;
    /// # use rac_rs::shared::ClientError;
    /// # let mut client = Client::new("", Default::default(), false);
    /// client.send_message("<{username}> Hello everyone!")?;
    /// # Ok::<(), ClientError>(())
    /// ```
    pub async fn send_message(&self, message: &str) -> Result<(), ClientError> {
        // Replacing the `{username}` placeholder with the actual username.
        let message = message.replace("{username}", &self.username);
        self.send_custom_message(&message).await
    }

    /// Sends a raw message to the server without any modifications.
    pub async fn send_custom_message(&self, message: &str) -> Result<(), ClientError> {
        let mut stream = self.get_stream().await?;

        // Sending the message to the RAC server.

        if self.password.is_some() {
            stream
                .write_all(
                    format!(
                        "\x02{}\n{}\n{}",
                        self.username,
                        self.password.as_deref().unwrap(),
                        message
                    )
                    .as_bytes(),
                )
                .await
                .map_err(ClientError::StreamWriteError)?;
            let mut buf = [0u8; 16];
            let n = stream
                .read(&mut buf)
                .await
                .map_err(ClientError::StreamReadError)?;
            if n == 0 {
                return Ok(());
            }
            return match buf[0] {
                0x01 => Err(ClientError::UserDoesNotExist),
                0x02 => Err(ClientError::IncorrectPassword),
                _ => Err(ClientError::UnexpectedResponse(
                    String::from_utf8_lossy(&buf[..n]).to_string(),
                )),
            };
        }

        // If user is not authorized, we can send the message directly, without an attempt to authorize.
        stream
            .write_all(format!("\x01{}", message).as_bytes())
            .await
            .map_err(ClientError::StreamWriteError)?;

        Ok(())
    }

    /// Resets the client's state to its default values.
    ///
    /// This clears the address, username, password, and message size.
    pub fn reset(&mut self) {
        self.current_messages_size = 0;
        self.address.clear();
        self.username.clear();
        self.password = None;
    }

    /// Returns the current size of messages known to the client.
    ///
    /// This value is updated after calls to `fetch_all_messages` or `fetch_new_messages`.
    pub fn current_messages_size(&self) -> usize {
        self.current_messages_size
    }

    /// Returns the current state of TLS usage.
    pub fn tls(&self) -> bool {
        self.use_tls
    }

    /// Returns a reference to the server address.
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Returns a reference to the client's username.
    pub fn username(&self) -> &str {
        &self.username
    }
}
