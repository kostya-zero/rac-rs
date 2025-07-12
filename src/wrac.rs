use crate::shared::{ClientError, Credentials};
use std::borrow::Cow;
use std::net::TcpStream;
use tungstenite::{Message, WebSocket, client::IntoClientRequest, connect, stream::MaybeTlsStream};

/// Concrete WebSocket stream type we deal with.
type WsStream = WebSocket<MaybeTlsStream<TcpStream>>;

/// A WebSocket client for interacting with a WRAC server.
///
/// The `WClient` provides methods to connect to a WRAC server over WebSockets.
///
/// # Example
///
/// ```no_run
/// use rac_rs::wrac::WClient;
/// use rac_rs::shared::Credentials;
///
/// # fn run() -> Result<(), rac_rs::shared::ClientError> {
/// let credentials = Credentials {
///     username: "test_user".to_string(),
///     password: Some("password123".to_string()),
/// };
///
/// let mut client = WClient::new(
///     "127.0.0.1:52666",
///     credentials,
///     false,
/// );
///
/// // Initialize connection before sending something
/// client.prepare()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct WClient {
    /// The current size of messages in the client.
    current_messages_size: usize,
    /// The address of the RAC server. Can be a full `ws(s)://` URL or just `host:port`.
    address: String,
    /// Whether to use TLS encryption (`wss://`).
    use_tls: bool,
    /// The username for authentication.
    username: String,
    /// The password for authentication, if required.
    password: Option<String>,
    /// Holds the WebSocket connection to WRAC.
    ws_connection: Option<WsStream>,
}

impl WClient {
    /// Creates a new `WClient` instance.
    ///
    /// # Arguments
    ///
    /// * `address` can be one of:
    ///   * a full URL (`ws://host:port/path`, `wss://host:port/path`)
    ///   * or just `host:port` (path defaults to `/`).
    /// * `credentials` - The username and optional password.
    /// * `use_tls` forces `wss://` when the input lacks a scheme.
    pub fn new(address: &str, credentials: Credentials, use_tls: bool) -> Self {
        Self {
            current_messages_size: 0,
            address: address.to_string(),
            use_tls,
            username: credentials.username,
            password: credentials.password,
            ws_connection: None,
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

    /// Turn the user‑supplied `address` into a valid WebSocket URL.
    fn build_url(&self) -> Result<String, ClientError> {
        if self.address.starts_with("ws://") || self.address.starts_with("wss://") {
            return Ok(self.address.to_string());
        }
        let scheme = if self.use_tls { "wss" } else { "ws" };
        Ok(format!("{scheme}://{}/", self.address))
    }

    /// Establishes a WebSocket connection to the RAC server.
    fn get_ws(&self) -> Result<WsStream, ClientError> {
        let url = self.build_url()?;
        let (ws, _resp) = connect(url.into_client_request().unwrap())
            .map_err(|e| ClientError::TlsInitializationError(e.to_string()))?;
        Ok(ws)
    }

    /// Initializes the connection to WRAC server.
    pub fn prepare(&mut self) -> Result<(), ClientError> {
        self.ws_connection = Some(self.get_ws()?);
        Ok(())
    }

    /// Checks if connection is established.
    fn check_connection(&self) -> Result<(), ClientError> {
        if self.ws_connection.is_none() {
            Err(ClientError::NoConnectionWRAC)
        } else {
            Ok(())
        }
    }

    /// Registers a new user on the WRAC server.
    ///
    /// # Errors
    ///
    /// Returns `ClientError::NoPassword` if no password specified for the client.
    /// Returns `ClientError::UsernameAlreadyTaken` if the username is already in use.
    /// Returns `ClientError::UnexpectedResponse` if got unexpected response from server.
    pub fn register_user(&mut self) -> Result<(), ClientError> {
        let mut ws = self.get_ws()?;
        if self.password.is_some() {
            let payload = format!(
                "\x03{}\n{}",
                self.username,
                self.password.as_deref().unwrap()
            );
            ws.send(Message::Binary(payload.into()))
                .map_err(|e| ClientError::WsSendError(e.to_string()))?;

            if let Ok(Message::Binary(buf)) = ws.read() {
                return match buf.first() {
                    Some(0x01) => Err(ClientError::UsernameAlreadyTaken),
                    Some(code) => Err(ClientError::UnexpectedResponse(format!("0x{code:02x}"))),
                    None => Ok(()),
                };
            }
            return Ok(());
        }
        Err(ClientError::NoPassword)
    }

    /// Fetches the total size of all messages on the server and updates the client's internal state.
    ///
    /// This is useful for determining the amount of data to fetch for all messages.
    pub fn fetch_messages_size(&mut self) -> Result<(), ClientError> {
        self.check_connection()?;
        let ws = self.ws_connection.as_mut().unwrap();
        ws.send(Message::Binary(vec![0x00].into()))
            .map_err(|e| ClientError::WsSendError(e.to_string()))?;
        let msg = ws
            .read()
            .map_err(|e| ClientError::WsReadError(e.to_string()))?;
        let txt = match msg {
            Message::Text(t) => t.to_string(),
            Message::Binary(b) => String::from_utf8_lossy(&b).into_owned(),
            _ => String::new(),
        };
        self.current_messages_size = txt
            .trim()
            .parse::<usize>()
            .map_err(|_| ClientError::ParseError("Failed to parse messages size".into()))?;
        Ok(())
    }

    /// Fetches all messages from the RAC server.
    ///
    /// This method retrieves all messages stored on the server and updates the
    /// client's internal message size tracker.
    pub fn fetch_all_messages(&mut self) -> Result<Vec<Cow<str>>, ClientError> {
        // Fetching new size explicitly
        self.fetch_messages_size()?;
        let ws = self.ws_connection.as_mut().unwrap();
        ws.send(Message::Binary(vec![0x01].into()))
            .map_err(|e| ClientError::WsSendError(e.to_string()))?;
        let all_msg = ws
            .read()
            .map_err(|e| ClientError::WsReadError(e.to_string()))?;
        let payload = match all_msg {
            Message::Text(t) => t.to_string(),
            Message::Binary(b) => String::from_utf8_lossy(&b).into_owned(),
            _ => String::new(),
        };
        Ok(payload
            .lines()
            .filter(|l| !l.is_empty())
            .map(|s| Cow::Owned(s.to_string()))
            .collect())
    }

    /// Fetches only new messages that have arrived since the last fetch.
    ///
    /// This method compares the current message size on the server with the client's
    /// stored size and retrieves only the difference. The client's internal message
    /// size tracker is updated upon successful fetch.
    pub fn fetch_new_messages(&mut self) -> Result<Vec<Cow<str>>, ClientError> {
        let old_size = self.current_messages_size;
        // Fetching new size via fetch_current_size because it's WRAC
        self.fetch_messages_size()?;
        let new_size = self.current_messages_size;
        if old_size >= new_size {
            return Ok(Vec::new());
        }
        // Because the first one will be closed after our request.
        let ws = self.ws_connection.as_mut().unwrap();
        ws.send(Message::Binary(
            format!("\x00\x02{}", self.current_messages_size).into(),
        ))
        .map_err(|e| ClientError::WsSendError(e.to_string()))?;
        let diff_msg = ws
            .read()
            .map_err(|e| ClientError::WsReadError(e.to_string()))?;
        let payload = match diff_msg {
            Message::Text(t) => t.to_string(),
            Message::Binary(b) => String::from_utf8_lossy(&b).into_owned(),
            _ => String::new(),
        };
        Ok(payload
            .lines()
            .filter(|l| !l.is_empty())
            .map(|s| Cow::Owned(s.to_string()))
            .collect())
    }

    /// Sends a message to the server.
    ///
    /// The placeholder `{username}` in the message will be replaced with the client's username.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use rac_rs::wrac::WClient;
    /// # use rac_rs::shared::{ClientError, Credentials};
    /// # async fn run() -> Result<(), ClientError> {
    /// # let mut client = WClient::new("", Default::default(), false);
    /// client.send_message("<{username}> Hello everyone!").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn send_message(&mut self, message: &str) -> Result<(), ClientError> {
        let msg = message.replace("{username}", &self.username);
        self.send_custom_message(&msg)
    }

    /// Sends a raw message to the server without any modifications.
    pub fn send_custom_message(&mut self, message: &str) -> Result<(), ClientError> {
        self.check_connection()?;
        let ws = self.ws_connection.as_mut().unwrap();
        if self.password.is_some() {
            let payload = format!(
                "\x02{}\n{}\n{}",
                self.username,
                self.password.as_deref().unwrap(),
                message
            );
            ws.send(Message::Binary(payload.into()))
                .map_err(|e| ClientError::WsSendError(e.to_string()))?;
            if let Ok(Message::Binary(buf)) = ws.read() {
                return match buf.first() {
                    Some(0x01) => Err(ClientError::UserDoesNotExist),
                    Some(0x02) => Err(ClientError::IncorrectPassword),
                    Some(code) => Err(ClientError::UnexpectedResponse(format!("0x{code:02x}"))),
                    None => Ok(()),
                };
            }
            return Ok(());
        }
        ws.send(Message::Binary(format!("\x01{}", message).into()))
            .map_err(|e| ClientError::WsSendError(e.to_string()))?;
        Ok(())
    }

    /// Resets the client's state to its default values and closes WebSocket connection.
    pub fn reset(&mut self) {
        self.current_messages_size = 0;
        self.address.clear();
        self.username.clear();
        self.password = None;
        self.use_tls = false;
        if let Some(ws) = &mut self.ws_connection {
            let _ = ws.close(None);
            self.ws_connection = None;
        }
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
