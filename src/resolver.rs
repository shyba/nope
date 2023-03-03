use std::net::SocketAddr;
use std::io;

pub struct DNSResolver {
    upstream: Vec<SocketAddr>,
    strategy: ResolutionStrategy
}

pub enum ResolutionStrategy {
    RoundRobin,
    All,
}

impl Default for DNSResolver {
    fn default() -> Self {
        Self { upstream: Default::default(), strategy: ResolutionStrategy::RoundRobin }
    }
}

impl DNSResolver {
    pub fn pick_upstream(&self, request_id: u16) -> &[SocketAddr] {
        match self.strategy {
            ResolutionStrategy::All => &self.upstream[..],
            ResolutionStrategy::RoundRobin => std::slice::from_ref(&self.upstream[request_id as usize % self.upstream.len()])
        }
    }
}