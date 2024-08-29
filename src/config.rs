use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use collie::{
    auth::model::database::keys_table,
    model::database::{self, feed_table, items_table, Connection},
};
use serde::Deserialize;

pub struct AppState {
    pub conn: Connection,
    pub config: Config,
}

impl AppState {
    pub fn new() -> Self {
        let config = read_config();
        Self {
            conn: open_connection(&config),
            config,
        }
    }
}

#[derive(Clone)]
pub struct SharedAppState {
    pub conn: Arc<Connection>,
    pub config: Config,
}

impl SharedAppState {
    pub fn new() -> Self {
        let config = read_config();
        Self {
            conn: Arc::new(open_connection(&config)),
            config,
        }
    }
}

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

fn from(path: &PathBuf) -> Config {
    let config = std::fs::read_to_string(path).expect("Failed to read config file.");
    toml::from_str(&config).expect("Failed to parse config file.")
}

fn read_config() -> Config {
    from(&PathBuf::from("data/config.toml"))
}

fn open_connection(config: &Config) -> Connection {
    let db = database::open_connection(&PathBuf::from(&config.database.path)).unwrap();

    let _ = database::Migration::new()
        .add_table(feed_table())
        .add_table(items_table())
        .add_table(keys_table())
        .migrate(&db);

    Connection { db: Mutex::new(db) }
}
