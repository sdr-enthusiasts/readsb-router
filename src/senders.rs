use crate::generics::reconnect_options;
use stubborn_io::StubbornTcpStream;
use tokio::sync::mpsc::Receiver;
use std::net::SocketAddr;
use stubborn_io::tokio::StubbornIo;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;


pub struct TCPSenderServer {
    host: SocketAddr,
    channel_to_receive_from: Receiver<String>,
}

impl TCPSenderServer {
    pub fn new(host: SocketAddr, channel_to_receive_from: Receiver<String>) -> Self {
        TCPSenderServer { host, channel_to_receive_from }
    }

    pub async fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut stream = match StubbornTcpStream::connect_with_options(self.host, reconnect_options()).await
        {
            Ok(stream) => stream,
            Err(e) => {
                error!("Error connecting to {}: {}", self.host, e);
                Err(e)?
            }
        };

        while let Some(message) = self.channel_to_receive_from.recv().await {
            match stream.write_all(message.as_bytes()).await {
                Ok(_) => (),
                Err(e) => {
                    error!("Error sending message to channel: {}", e);
                    return Err(e)?
                }
            }
        }

        Ok(())
    }
}