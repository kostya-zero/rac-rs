//! `rac_rs` is a Rust implementation of a client for RAC (Real Address Chat) protocol.
//!
//! This library provides a `Client` to interact with RAC servers, allowing you to:
//! - Connect to a server.
//! - Send and receive messages.
//! - Register new users.
//!
//! It supports both `RAC` (v1.99.x) and `RACv2` (v2.x with authentication) protocols.
//!
//! Also, library provides an async client implementation when the `async_client` feature is enabled.
//!
//! # Example
//!
//! ```no_run
//! use rac_rs::client::Client;
//! use rac_rs::shared::{Connection, Credentials};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let credentials = Credentials {
//!         username: "test_user".to_string(),
//!         password: Some("password123".to_string()),
//!     };
//!
//!     let mut client = Client::new(
//!         "127.0.0.1:42666".to_string(),
//!         credentials,
//!         Connection::RACv2,
//!     );
//!
//!     // Test the connection
//!     client.test_connection()?;
//!
//!     // Register a new user (for RACv2)
//!     // client.register_user()?;
//!
//!     // Send a message
//!     client.send_message("<{username}> Hello everyone!")?;
//!
//!     // Fetch all messages
//!     let messages = client.fetch_all_messages()?;
//!     for msg in messages {
//!         println!("{}", msg);
//!     }
//!
//!     Ok(())
//! }
//! ```

/// Contains the client implementation for interacting with RAC servers.
pub mod client;

/// Contains the async client implementation for interacting with RAC servers.
#[cfg(feature = "async_client")]
pub mod async_client;

/// Contains shared type and utilities that's used across the library.
pub mod shared;
