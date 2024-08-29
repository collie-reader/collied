use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub stage: String,
    pub database: DatabaseConfig,
    pub producer: ProducerConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProducerConfig {
    pub polling_frequency: u64,
    pub proxy: Option<String>,
}

pub fn from(path: &PathBuf) -> Config {
    let config = std::fs::read_to_string(path).expect("Failed to read config file.");
    toml::from_str(&config).expect("Failed to parse config file.")
}
