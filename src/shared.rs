use thiserror::Error;

/// Represents errors that can occur while interacting with the RAC server.
#[derive(Error, Debug)]
pub enum ClientError {
    /// Failed to establish a TCP connection to the RAC server.
    #[error("Failed to connect to the RAC server: {0}")]
    ConnectionError(std::io::Error),

    /// Failed to write data to the TCP stream.
    #[error("Failed to write data to the stream: {0}")]
    StreamWriteError(std::io::Error),

    /// Failed to read data from the TCP stream.
    #[error("Failed to read data from the stream: {0}")]
    StreamReadError(std::io::Error),

    /// Failed to read a message to the server via WebSocket.
    #[error("Failed to read message via WebSocket: {0}")]
    WsReadError(String),

    /// Failed to send a message to the server via WebSocket.
    #[error("Failed to send message  via WebSocket: {0}")]
    WsSendError(String),

    /// Failed to parse data received from the server.
    #[error("Failed to parse data: {0}")]
    ParseError(String),
    
    /// The server closed the connection while sending a packet.
    #[error("Server closed the connection while sending a packet")]
    ServerClosedConnection,
    
    /// The specified user does not exist on the server (RACv2 only).
    #[error("User does not exist on the server")]
    UserDoesNotExist,

    /// The provided password is incorrect (RACv2 only).
    #[error("Incorrect password")]
    IncorrectPassword,

    /// Received an unexpected response from the server.
    #[error("Unexpected response from the server: {0}")]
    UnexpectedResponse(String),

    /// The username is already taken during registration (RACv2 only).
    #[error("Username is already taken")]
    UsernameAlreadyTaken,

    /// An operation was attempted that is not supported by the current connection type.
    /// For example, trying to register a user with a `RAC` connection.
    #[error("No password specified.")]
    NoPassword,

    /// An error that occurs while initializing a TLS connection.
    #[error("Failed to initialize TLS connection: {0}")]
    TlsInitializationError(String),

    /// An error that occurs when connection to WRAC server is not established first.
    #[error("Not connected to WRAC. Establish connection first.")]
    NoConnectionWRAC,
}

/// Represents the credentials required to connect to a RAC server.
#[derive(Debug, Clone, Default)]
pub struct Credentials {
    /// The username for authentication.
    pub username: String,
    /// The password for authentication. This is only used for `RACv2` connections.
    pub password: Option<String>,
}
