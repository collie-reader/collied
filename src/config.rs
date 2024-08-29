use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub stage: String,
    pub database: DatabaseConfig,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
}

pub fn from(path: &PathBuf) -> Config {
    let config = std::fs::read_to_string(path).expect("Failed to read config file.");
    toml::from_str(&config).expect("Failed to parse config file.")
}
