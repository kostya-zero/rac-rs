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

    /// Failed to parse data received from the server.
    #[error("Failed to parse data: {0}")]
    ParseError(String),

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
    #[error("Incorrect connection type")]
    IncorrectConnectionType,
}

/// Represents the type of connection to a RAC server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Connection {
    /// Represents a connection to a RAC server using the legacy v1.99.x protocol.
    /// This protocol does not support authentication.
    RAC,
    /// Represents a connection to a RAC server using the v2.x protocol.
    /// This protocol adds support for user registration and authentication.
    RACv2,
}

/// Represents the credentials required to connect to a RAC server.
#[derive(Debug, Clone, Default)]
pub struct Credentials {
    /// The username for authentication.
    pub username: String,
    /// The password for authentication. This is only used for `RACv2` connections.
    pub password: Option<String>,
}