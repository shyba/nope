use std::{io, net::SocketAddr};

use futures::future::join_all;
use tokio::net::UdpSocket;

pub struct DNSResolver {
    upstream: Vec<SocketAddr>,
    strategy: ResolutionStrategy,
    counter: u16,
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
    pub fn pick_upstream(&self, request_id: u16) -> &[SocketAddr] {
        match self.strategy {
            ResolutionStrategy::All => &self.upstream[..],
            ResolutionStrategy::RoundRobin => {
                std::slice::from_ref(&self.upstream[request_id as usize % self.upstream.len()])
            }
        }
    }

    pub async fn send_request(&mut self, req: &[u8], socket: UdpSocket) -> Result<(), io::Error> {
        self.counter = self.counter.wrapping_add(1);
        let futures: Vec<_> = self
            .pick_upstream(self.counter)
            .iter()
            .map(|upstream| socket.send_to(req, upstream))
            .collect();
        join_all(futures).await;
        Ok(())
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
            counter: 0,
        };
        assert_eq!(&[upstream[0]], resolver.pick_upstream(0));
        assert_eq!(&[upstream[1]], resolver.pick_upstream(1));
        assert_eq!(&[upstream[0]], resolver.pick_upstream(2));
    }

    #[test]
    fn test_all() {
        let upstream = vec!["1.1.1.1:53".parse().unwrap(), "1.0.0.1:53".parse().unwrap()];
        let resolver = DNSResolver {
            upstream: upstream.clone(),
            strategy: ResolutionStrategy::All,
            counter: 0,
        };
        assert_eq!(&upstream[..], resolver.pick_upstream(0));
        assert_eq!(&upstream[..], resolver.pick_upstream(1));
        assert_eq!(&upstream[..], resolver.pick_upstream(2));
    }
}
