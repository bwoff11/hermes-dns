use crate::{dns::Message as DNSMessage, settings::CacheSettings};

pub struct Cache {
    enabled: bool,
    size: usize,
}

impl Cache {
    pub fn new(settings: &CacheSettings) -> Self {
        Cache {
            enabled: settings.enabled,
            size: settings.size,
        }
    }

    pub fn query(&self, _domain: &str) -> Option<DNSMessage> {
        None
    }
}