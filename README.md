# rac-rs

[//]: # ( FIXME: Uncomment this when it will be published on crates.io)
[//]: # ([![crates.io]&#40;https://img.shields.io/crates/v/rac_rs.svg&#41;]&#40;https://crates.io/crates/rac_rs&#41;)
[//]: # ([![docs.rs]&#40;https://docs.rs/rac_rs/badge.svg&#41;]&#40;https://docs.rs/rac_rs&#41;)

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](/LICENSE)


A Rust client library for RAC (Real Address Chat) protocol.

`rac_rs` provides both synchronous and asynchronous clients to interact with RAC servers. It supports both the legacy
`RAC` (v1.99.x) protocol and the modern `RACv2` (v2.x with authentication) protocol.

## Features

- Support for both `RAC` and `RACv2` protocols.
- Synchronous (`Client`) and Asynchronous (`async_client`) APIs.
- User registration and authentication for `RACv2`.
- Fetch all or only new messages.
- Send messages with `{username}` placeholder replacement.
- Comprehensive error handling via `ClientError`.

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

## API Overview

- **`client::Client`**: The main entry point for interacting with a RAC server using a synchronous API.
- **`async_client::Client`**: The async version of the `Client`. Requires the `async_client` feature to be enabled.
- **`shared::Connection`**: An enum to specify the protocol version:
    - `Connection::RAC`: For legacy v1.99.x servers without authentication.
    - `Connection::RACv2`: For v2.x servers with authentication support.
- **`shared::Credentials`**: A struct to hold the `username` and optional `password` for connecting.
- **`shared::ClientError`**: An enum representing all possible errors that can occur, such as connection issues, parsing
  errors, or authentication failures.

## Async Support

This library provides an asynchronous client under the `rac_rs::async_client` module. The API is very similar to the
synchronous client but uses `async/await`. You will need to enable the `async_client` feature for `rac_rs in your `
Cargo.toml`.

### Async Example

First, add `async_client` feature to the features field in your `Cargo.toml`:

```toml
[dependencies]
rac_rs = { version = "0.1.0", features = ["async_client"] }
```

Then, you can use the async client:

```rust
use rac_rs::async_client::Client; // Note: from async_client module
use rac_rs::shared::{Connection, Credentials};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let credentials = Credentials {
        username: "async_user".to_string(),
        password: Some("password123".to_string()),
    };

    let mut client = Client::new(
        "127.0.0.1:42666".to_string(),
        credentials,
        Connection::RACv2,
    );

    // All client methods are async
    client.test_connection().await?;
    println!("Async connection successful.");

    client.send_message("<{username}> Hello from asynchronous rac_rs!").await?;
    println!("Async message sent.");

    let messages = client.fetch_all_messages().await?;
    println!("\n--- Fetched Messages (Async) ---");
    for msg in messages {
        println!("{}", msg);
    }

    Ok(())
}
```

## License

This project is licensed under the MIT License.
