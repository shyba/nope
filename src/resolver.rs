use std::net::SocketAddr;

pub struct DNSResolver {
    upstream: Vec<SocketAddr>,
    strategy: ResolutionStrategy,
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
        };
        assert_eq!(&[upstream[0]], resolver.pick_upstream(0));
        assert_eq!(&[upstream[1]], resolver.pick_upstream(1));
        assert_eq!(&[upstream[0]], resolver.pick_upstream(2));
    }
}
