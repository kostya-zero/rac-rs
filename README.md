# `rac-rs`

[//]: # ( FIXME: Uncomment this when it will be published on crates.io)
[//]: # ([![crates.io]&#40;https://img.shields.io/crates/v/rac_rs.svg&#41;]&#40;https://crates.io/crates/rac_rs&#41;)
[//]: # ([![docs.rs]&#40;https://docs.rs/rac_rs/badge.svg&#41;]&#40;https://docs.rs/rac_rs&#41;)

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](/LICENSE)


A Rust client library for RAC (Real Address Chat) protocol.

`rac_rs` provides both synchronous and asynchronous clients to interact with RAC servers. It supports both the TCP
`RAC` protocol and the WebSocket-based `WRAC` protocol.

## Features

- Support for both `RAC` and `WRAC` protocols.
- TLS support for secure connections.
- Synchronous (`Client`) and Asynchronous (`async_client`) APIs.
- User registration and authentication for `RACv2`.
- Fetch all or only new messages.
- Send messages with `{username}` placeholder replacement.
- Comprehensive error handling via `ClientError`.

> [!WARNING]
> The WRAC protocol implementation is unstable and may not work correctly. 
> Proceed with caution if you plan to use it.

> [!NOTE]
> This library is still in development and may not cover all edge cases or features of the RAC protocol. Contributions
> are welcome!

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rac_rs = "0.1.0"
```

Or use bleeding-edge version from GitHub:

```toml
[dependencies]
rac_rs = { git = "https://github.com/kostya-zero/rac-rs.git" }
```

## Configuring

The crate APIs are split into separate features:

- `client` - Synchronous client for RAC protocol.
- `async_client` - Asynchronous client for RAC protocol.
- `wrac` - Synchronous client for WRAC protocol.
- `async_wrac` - Asynchronous client for WRAC protocol.

All of these features are enabled by default.

## Usage

Here is a basic example of how to use the synchronous `Client`.

```rust
use rac_rs::client::Client;
use rac_rs::shared::{ClientError, Connection, Credentials};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // For RACv2 connections with authentication
    let credentials = Credentials {
        username: "test_user".to_string(),
        password: Some("password123".to_string()),
    };

    let mut client = Client::new(
        "127.0.0.1:42666".to_string(), // Your RAC server address
        credentials,
        Connection::RACv2,
    );

    // For legacy RAC connections (no authentication)
    // let mut client = Client::new(
    //     "127.0.0.1:42666".to_string(),
    //     Credentials { username: "guest".to_string(), password: None },
    //     Connection::RAC,
    // );

    // Test the connection
    client.test_connection()?;
    println!("Successfully connected to the server.");

    // Register a new user (only for RACv2)
    // This might fail if the user already exists.
    match client.register_user() {
        Ok(()) => println!("User registered successfully!"),
        Err(ClientError::UsernameAlreadyTaken) => println!("Username is already taken."),
        Err(e) => println!("Failed to register user: {}", e),
    }

    // Send a message. The `{username}` placeholder is automatically replaced.
    client.send_message("<{username}> Hello from rac_rs!")?;
    println!("Message sent.");

    // Fetch all messages
    println!("\n--- All Messages ---");
    let messages = client.fetch_all_messages()?;
    for msg in &messages {
        println!("{}", msg);
    }

    // Fetch only new messages since the last fetch
    println!("\n--- New Messages ---");
    // In a real application, you might wait here for new messages to arrive.
    let new_messages = client.fetch_new_messages()?;
    if new_messages.is_empty() {
        println!("No new messages.");
    } else {
        for msg in new_messages {
            println!("{}", msg);
        }
    }

    Ok(())
}
```

## Projects using `rac_rs`:
- [Tower](https://github.com/kostya-zero/tower): A modern desktop client for RAC protocol built with Tauri.

## License

This project is licensed under the [MIT License](LICENSE).
