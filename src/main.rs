use std::{env, io};
use tokio::net::UdpSocket;
use std::net::SocketAddr;
use std::collections::HashMap;

const MAX_UDP_SIZE: usize = 1<<16;

struct DNSForwarderServer {
    socket: UdpSocket,
    upstream: SocketAddr,
}

impl DNSForwarderServer {
    async fn run(self) -> Result<(), io::Error> {
        let mut buf = [0; MAX_UDP_SIZE];
        let mut id: u16 = 0;
        let mut pending: HashMap<u16, (SocketAddr, u16)> = HashMap::new();
        loop {
            let (size, origin) = self.socket.recv_from(&mut buf).await?;
            println!("Received {} bytes from {:?}", size, origin);
            if size < 8 {
                println!("Short invalid request from {:?}", origin);
                continue;
            }
            let original_id = u16::from_be_bytes(buf[0..2].try_into().expect("should never fail"));
            if origin == self.upstream {
                println!("origin is upstream");
                // response from upstream
                if let Some((client, client_id)) = pending.remove(&original_id) {
                    println!("Reply to {}", client);
                    buf[0..2].copy_from_slice(&client_id.to_be_bytes());
                    let sent = self.socket.send_to(&buf[..size], client).await?;
                    if sent != size {
                        println!("Incomplete send, only {}/{} written", sent, size);
                    }
                }
                continue;
            }
            println!("sending upstream {} {} bytes", self.upstream, size);
            buf[0..2].copy_from_slice(&id.to_be_bytes());
            let sent = self.socket.send_to(&buf[..size], self.upstream).await?;
            if sent != size {
                println!("Incomplete send, only {}/{} written", sent, size);
                continue;
            }
            println!("Request id {} from {}", original_id, origin);
            pending.insert(id, (origin, original_id));
            id = id.wrapping_add(1);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:3030".to_string());

    let socket = UdpSocket::bind(&addr).await?;
    println!("Listening on: {}", socket.local_addr()?);

    DNSForwarderServer {
        socket,
        upstream: "1.1.1.1:53".parse().expect("invalid upstream"),
    }.run().await
}