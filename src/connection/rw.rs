use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::{Connection, Stream};

impl Connection {
    // Method to read from the stream
    pub async fn read(&mut self, buf: &mut [u8]) -> tokio::io::Result<usize> {
        match self.stream {
            Stream::Plain(ref mut stream) => stream.read(buf).await,
            Stream::Encrypted(ref mut stream) => stream.read(buf).await,
        }
    }

    // Method to write to the stream
    pub async fn write(&mut self, buf: &[u8]) -> tokio::io::Result<()> {
        match &mut self.stream {
            Stream::Plain(stream) => stream.write_all(buf).await,
            Stream::Encrypted(stream) => stream.write_all(buf).await,
        }
    }
}
