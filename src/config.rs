use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use collie::{
    auth::repository::database::keys_table,
    auth::service::key,
    repository::database::{self, feeds_table, items_table, DbConnection},
};
use serde::Deserialize;

pub struct Context {
    pub conn: DbConnection,
    pub config: Config,
    pub server_secret: String,
}

impl Context {
    pub fn new(config_path: Option<&Path>) -> Self {
        let config = read_config(config_path);
        Self {
            conn: open_connection(&config),
            config,
            server_secret: key::generate(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub stage: String,
    pub database: DatabaseConfig,
    pub producer: ProducerConfig,
    pub daemon: DaemonConfig,
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

#[derive(Debug, Deserialize, Clone)]
pub struct DaemonConfig {
    pub pid_file: String,
    pub error_log: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            stage: "production".to_string(),
            database: DatabaseConfig {
                path: "/etc/collied/collie.db".to_string(),
            },
            producer: ProducerConfig {
                polling_frequency: 600,
                proxy: None,
            },
            daemon: DaemonConfig {
                pid_file: "/tmp/collied.pid".to_string(),
                error_log: None,
            },
        }
    }
}

fn read_config(path: Option<&Path>) -> Config {
    let config = match path {
        Some(path) => fs::read_to_string(path).unwrap(),
        None => fs::read_to_string("config.toml")
            .unwrap_or(fs::read_to_string("/etc/collied/config.toml").unwrap()),
    };

    toml::from_str(&config).expect("Failed to parse config file.")
}

fn open_connection(config: &Config) -> DbConnection {
    let db = database::open_connection(&PathBuf::from(&config.database.path)).unwrap();

    let _ = database::Migration::new()
        .table(feeds_table())
        .table(items_table())
        .table(keys_table())
        .migrate(&db);

    Arc::new(Mutex::new(db))
}
