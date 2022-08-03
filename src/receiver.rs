use std::net::SocketAddr;
use stubborn_io::StubbornTcpStream;
use tokio::sync::mpsc::Sender;
use tokio_util::codec::{Framed, LinesCodec};
use tokio_stream::StreamExt;

use crate::generics::reconnect_options;

pub struct TCPReceiverServer {
    host: SocketAddr,
    channel_to_send_to: Sender<String>,
}

impl TCPReceiverServer {
    pub fn new(host: SocketAddr, channel_to_send_to: Sender<String>) -> Self {
        TCPReceiverServer { host, channel_to_send_to }
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let stream = match StubbornTcpStream::connect_with_options(self.host, reconnect_options()).await
        {
            Ok(stream) => stream,
            Err(e) => {
                error!("Error connecting to {}: {}", self.host, e);
                Err(e)?
            }
        };

        // create a buffered reader and send the messages to the channel

        let reader = tokio::io::BufReader::new(stream);
        let mut lines = Framed::new(reader, LinesCodec::new());

        while let Some(Ok(line)) = lines.next().await {
            match self.channel_to_send_to.send(line).await {
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
