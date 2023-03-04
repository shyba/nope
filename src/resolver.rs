use std::{io, net::SocketAddr, sync::atomic::AtomicU16};

use futures::future::join_all;
use log::debug;
use tokio::net::UdpSocket;

pub struct DNSResolver {
    upstream: Vec<SocketAddr>,
    strategy: ResolutionStrategy,
    counter: AtomicU16,
}

pub enum ResolutionStrategy {
    RoundRobin,
    All,
}

impl Default for DNSResolver {
    fn default() -> Self {
        Self {
            upstream: Default::default(),
            strategy: ResolutionStrategy::RoundRobin,
            counter: Default::default(),
        }
    }
}

impl DNSResolver {
    pub fn produce_upstreams(&self) -> &[SocketAddr] {
        let current = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        match self.strategy {
            ResolutionStrategy::All => &self.upstream[..],
            ResolutionStrategy::RoundRobin => {
                std::slice::from_ref(&self.upstream[current as usize % self.upstream.len()])
            }
        }
    }

    pub async fn send_request(
        &mut self,
        req: &mut [u8],
        socket: &UdpSocket,
    ) -> Result<(), io::Error> {
        let futures: Vec<_> = self
            .produce_upstreams()
            .iter()
            .map(|target| Self::do_send(req, &socket, target))
            .collect();
        join_all(futures).await;
        Ok(())
    }

    async fn do_send(req: &[u8], socket: &UdpSocket, target: &SocketAddr) -> io::Result<usize> {
        debug!("sending {} bytes to {}", req.len(), target);
        socket.send_to(req, target).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_round_robin() {
        let upstream = vec!["1.1.1.1:53".parse().unwrap(), "1.0.0.1:53".parse().unwrap()];
        let resolver = DNSResolver {
            upstream: upstream.clone(),
            strategy: ResolutionStrategy::RoundRobin,
            counter: AtomicU16::new(0),
        };
        assert_eq!(&[upstream[0]], resolver.produce_upstreams());
        assert_eq!(&[upstream[1]], resolver.produce_upstreams());
        assert_eq!(&[upstream[0]], resolver.produce_upstreams());
    }

    #[test]
    fn test_all() {
        let upstream = vec!["1.1.1.1:53".parse().unwrap(), "1.0.0.1:53".parse().unwrap()];
        let resolver = DNSResolver {
            upstream: upstream.clone(),
            strategy: ResolutionStrategy::All,
            counter: AtomicU16::new(0),
        };
        assert_eq!(&upstream[..], resolver.produce_upstreams());
        assert_eq!(&upstream[..], resolver.produce_upstreams());
        assert_eq!(&upstream[..], resolver.produce_upstreams());
    }
}
