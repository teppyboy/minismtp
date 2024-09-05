mod connection;
mod parser;

/**
Contains the SmtpServer struct and its implementation.
*/
pub mod server;

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use crate::server::SmtpServer;
    use async_smtp::{Envelope, SendableEmail, SmtpClient, SmtpTransport};
    use lettre::message::header::ContentType;
    use lettre::transport::smtp::client::{Tls, TlsParameters};
    use lettre::{Message, Transport};
    use tokio::{io::BufStream, net::TcpStream};

    async fn send_email_async_smtp() {
        let stream = BufStream::new(TcpStream::connect("localhost:2525").await.unwrap());
        let client = SmtpClient::new();
        let mut transport = SmtpTransport::new(client, stream).await.unwrap();

        let email = SendableEmail::new(
            Envelope::new(
                Some("user@localhost".parse().unwrap()),
                vec!["root@localhost".parse().unwrap()],
            )
            .unwrap(),
            "Hello world",
        );
        transport.send(email).await.unwrap();
        transport.quit().await.unwrap();
    }

    fn send_email_lettre() {
        let email = Message::builder()
            .from("NoBody <nobody@domain.tld>".parse().unwrap())
            .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
            .to("Hei <hei@domain.tld>".parse().unwrap())
            .subject("Happy new year")
            .header(ContentType::TEXT_PLAIN)
            .body(String::from("Be happy!"))
            .unwrap();
        let tls_parameters = TlsParameters::builder("localhost".to_string())
            .dangerous_accept_invalid_certs(true)
            .dangerous_accept_invalid_hostnames(true)
            .build()
            .unwrap();
        // Open a remote connection to gmail
        let mailer = lettre::SmtpTransport::builder_dangerous("localhost")
            .tls(Tls::Required(tls_parameters))
            .port(2525)
            .build();

        mailer.send(&email).unwrap();
        drop(mailer);
    }

    #[tokio::test]
    async fn test() {
        env_logger::builder().is_test(true).try_init().unwrap();

        let server = SmtpServer::new(
            "localhost",
            2525,
            "localhost",
            Some(Duration::from_secs(10)),
            None,
            Some("cert.pem".into()),
            Some("key.pem".into()),
        );

        let listening_server = server.start().await.unwrap();

        log::info!("Sending via async-smtp");
        let _ = tokio::spawn(send_email_async_smtp()).await;
        listening_server.mail_rx.recv().await.unwrap();

        log::info!("Sending via lettre");

        thread::spawn(|| send_email_lettre());
        let mail = listening_server.mail_rx.recv().await.unwrap();
        log::info!("Received mail: {:?}", mail);
        listening_server.stop().await.unwrap();
    }
}
