use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct ConfigSettings {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
}

pub fn load_config() -> Result<ConfigSettings, config::ConfigError> {
    let builder = Config::builder()
        .add_source(File::with_name("config"))
        .add_source(Environment::with_prefix("APP").separator("__"));

    let config = builder.build()?;
    config.try_deserialize::<ConfigSettings>()
}
