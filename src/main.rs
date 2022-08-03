#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;
extern crate stubborn_io;
extern crate tokio;
extern crate tokio_stream;
extern crate tokio_util;

#[path = "./config_options.rs"]
mod config_options;

#[path = "./receiver.rs"]
mod receiver;

#[path = "./generics.rs"]
mod generics;

#[path = "./senders.rs"]
mod senders;

use crate::config_options::{Input, SetupLogging};
use crate::senders::TCPSenderServer;
use chrono::Local;
use clap::Parser;
use env_logger::Builder;
use std::io::Write;
use tokio::sync::mpsc;
use std::net::SocketAddr;
use tokio::sync::mpsc::Sender;

#[tokio::main]
async fn main() {
    let args: Input = Input::parse();

    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, args.verbose.set_logging_level())
        .init();

    let (tx_from_receivers, mut rx_from_receivers) = mpsc::channel(32);
    let mut sender_hosts: Vec<Sender<String>> = Vec::new();

    for host in args.get_readsb.iter() {
        let host_socket: SocketAddr;

        match host.parse::<SocketAddr>() {
            Ok(addr) => host_socket = addr,
            Err(e) => {
                error!("{}", e);
                continue;
            }
        }

        let tx_to_receivers = tx_from_receivers.clone();
        let receiver = receiver::TCPReceiverServer::new(host_socket, tx_to_receivers);

        tokio::spawn(async move {
            if let Err(e) = receiver.run().await {
                error!("Error in receiver: {}", e);
            }
        });
    }

    for host in args.send_readsb.iter() {
        let host_socket: SocketAddr;

        match host.parse::<SocketAddr>() {
            Ok(addr) => host_socket = addr,
            Err(e) => {
                error!("{}", e);
                continue;
            }
        }

        let (tx_receivers, rx_receivers) = mpsc::channel(32);
        let sender = TCPSenderServer::new(host_socket, rx_receivers);

        tokio::spawn(async move {
            if let Err(e) = sender.run().await {
                error!("Error in sender: {}", e);
            }
        });

        sender_hosts.push(tx_receivers);
    }

    while let Some(message) = rx_from_receivers.recv().await {
        for sender in sender_hosts.iter_mut() {
            match sender.send(message.clone()).await {
                Ok(_) => (),
                Err(e) => {
                    error!("Error sending message to channel: {}", e);
                    return;
                }
            }
        }
    }
}
