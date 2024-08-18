mod connection;
mod load;
mod parser;
pub mod server;



#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::server::SmtpServer;

    #[tokio::test]
    async fn test() {
        env_logger::builder().is_test(true).try_init().unwrap();
        let server = SmtpServer::new(
            "localhost",
            2525,
            "localhost",
            Some(Duration::from_secs(100)),
            None,
            Some("cert.pem".into()),
            Some("key.pem".into()),
        );

        server.start().await.unwrap();

        tokio::time::sleep(Duration::from_secs(6000)).await;
    }
}
