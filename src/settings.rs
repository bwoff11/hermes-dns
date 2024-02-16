use config::Config;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub listeners: ListenersSettings,
    pub resolver: ResolverSettings,
}

#[derive(Debug, Deserialize)]
pub struct ListenersSettings {
    pub udp: ListenerSettings,
    pub tcp: ListenerSettings,
}

#[derive(Debug, Deserialize)]
pub struct ListenerSettings {
    pub enabled: bool,
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct ResolverSettings {
    pub cache: CacheSettings,
    pub upstreams: Vec<UpstreamSettings>,
}

#[derive(Debug, Deserialize)]
pub struct CacheSettings {
    pub enabled: bool,
    pub size: usize,
}

#[derive(Debug, Deserialize)]
pub struct UpstreamSettings {
    pub address: String,
    pub port: u16,
    pub protocol: String,
}

impl Settings {
    pub fn load() -> Result<Self, config::ConfigError> {
        let settings = Config::builder()
            .add_source(config::File::with_name("/etc/hermes-dns/settings.toml"))
            .build()?;
        settings.try_deserialize()
    }
}
