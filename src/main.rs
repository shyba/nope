use std::{env, io};
use tokio::net::UdpSocket;
use std::net::SocketAddr;
use std::collections::HashMap;

const MAX_UDP_SIZE: usize = 1<<16;

struct DNSForwarderServer {
    socket: UdpSocket,
    upstream: SocketAddr,
    pending: HashMap<(SocketAddr, u16), SocketAddr>
}

impl DNSForwarderServer {
    async fn run(self) -> Result<(), io::Error> {
        let mut to_send = None;
        let mut buf = [0; MAX_UDP_SIZE];

        loop {
            if let Some((size, peer)) = to_send {
                let amt = self.socket.send_to(&buf[..size], self.upstream).await?;

                println!("Sent {}/{} bytes to {}", amt, size, peer);
            }

            to_send = Some(self.socket.recv_from(&mut buf).await?);
        }
    }

    async fn receiver(self) -> Result<(), io::Error> {
        let mut buf = [0; MAX_UDP_SIZE];
        loop {
            let (size, origin) = self.socket.recv_from(&mut buf).await?;

        }
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:3030".to_string());

    let socket = UdpSocket::bind(&addr).await?;
    println!("Listening on: {}", socket.local_addr()?);

    DNSForwarderServer {
        socket,
        upstream: "1.1.1.1:53".parse().expect("invalid upstream"),
        pending: HashMap::new()
    }.run().await
}