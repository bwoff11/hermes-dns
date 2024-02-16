use crate::settings::UpstreamSettings;

pub struct Upstreams {
    upstreams: Vec<Upstream>,
}

impl Upstreams {
    pub fn new(upstream_settings: &[UpstreamSettings]) -> Self {
        let upstreams = upstream_settings
            .iter()
            .map(|us| Upstream {
                address: us.address.clone(),
            })
            .collect();

        Self { upstreams }
    }

    pub fn query(&self, _domain: &str) -> Option<String> {
        None
    }
}

struct Upstream {
    address: String,
}