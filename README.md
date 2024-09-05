# `minismtp` - A light RFC-compliant* SMTP server library for Rust.

[![crates.io](https://img.shields.io/crates/v/minismtp.svg)](https://crates.io/crates/minismtp)
[![Documentation](https://docs.rs/minismtp/badge.svg)](https://docs.rs/minismtp)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)


This is the implementation of a light SMTP server library for Rust. It is designed to be used in a server application that needs to receive emails.

It has a fully custom-built SMTP command parser and handler. The most advanced SMTP server for rust so far has been [Stalwart's SMTP Server](https://github.com/stalwartlabs/smtp-server), which is a great library but I believe there exists use cases where you just want something minimal and simple.

Due to time restrictions I have restricted the scope of this SMTP server to serve as an [MTA](https://en.wikipedia.org/wiki/Message_transfer_agent) only. This means that it does not perform any kind of processing on the received emails, it just receives them and transmits them via an [unbounded](https://docs.rs/async-std/latest/async_std/channel/fn.unbounded.html) channel. It is up to the user to perform security checks like [SPF](https://en.wikipedia.org/wiki/Sender_Policy_Framework) or [DKIM](https://en.wikipedia.org/wiki/DomainKeys_Identified_Mail) verification.

## Recognized SMTP commands
- `HELO` - HELO
- `EHLO` - Extended HELO
- `STARTTLS` - Supports upgrading to TLS
- `MAIL FROM` - Sender email address
- `RCPT TO` - Recipient email address
- `DATA` - Email data
- `QUIT` - Close connection

## Encryption
The server supports full encryption via the `STARTTLS` command. The encryption upgrade is performed through my [tokio-tls-upgrade](https://crates.io/crates/tokio-tls-upgrade) which is a custom-built library that allows for a seamless upgrade of a TCP connection to a TLS connection.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
minismtp = "0.1.0"
```

Here is an example of how to use the library:

```rust
use minismtp::server::SmtpServer;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::main]
async fn main() {
    let server = SmtpServer::new(
        "localhost",
        2525,
        "localhost",
        Some(Duration::from_secs(10)),
        None,
        None,
        None,
    );

    let listening_server = server.start().await.unwrap();

    // Actually send an email to the server and do something with this
    // returned value.
    let _ = timeout(Duration::from_secs(5), listening_server.mail_rx.recv()).await;

    listening_server.stop().await.unwrap();
}
```

## License

See the [LICENSE](LICENSE) file for license rights and limitations (MIT).