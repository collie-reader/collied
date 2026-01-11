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
use sea_query::{ColumnDef, Iden, Table, TableStatement};
use serde::Deserialize;

#[derive(Iden)]
pub enum Settings {
    Table,
    Key,
    Value,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    Read(#[from] std::io::Error),
    #[error("Failed to parse config file: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("Failed to open database: {0}")]
    Database(String),
}

pub struct Context {
    pub conn: DbConnection,
    pub config: Config,
    pub server_secret: String,
}

impl Context {
    pub fn new(config_path: Option<&Path>) -> Result<Self, ConfigError> {
        let config = read_config(config_path)?;
        let conn = open_connection(&config)?;
        let server_secret = load_or_generate_secret(&conn)?;
        Ok(Self {
            conn,
            config,
            server_secret,
        })
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

pub fn settings_table() -> Vec<TableStatement> {
    let create_stmt = Table::create()
        .table(Settings::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(Settings::Key)
                .text()
                .not_null()
                .primary_key(),
        )
        .col(ColumnDef::new(Settings::Value).text().not_null())
        .to_owned();

    vec![TableStatement::Create(create_stmt)]
}

fn read_config(path: Option<&Path>) -> Result<Config, ConfigError> {
    let config_str = match path {
        Some(path) => Some(fs::read_to_string(path)?),
        None => fs::read_to_string("config.toml")
            .or_else(|_| fs::read_to_string("/etc/collied/config.toml"))
            .ok(),
    };

    match config_str {
        Some(s) => Ok(toml::from_str(&s)?),
        None => Ok(Config::default()),
    }
}

fn open_connection(config: &Config) -> Result<DbConnection, ConfigError> {
    let db = database::open_connection(&PathBuf::from(&config.database.path))
        .map_err(|e| ConfigError::Database(e.to_string()))?;

    let _ = database::Migration::new()
        .table(feeds_table())
        .table(items_table())
        .table(keys_table())
        .table(settings_table())
        .migrate(&db);

    Ok(Arc::new(Mutex::new(db)))
}

fn load_or_generate_secret(conn: &DbConnection) -> Result<String, ConfigError> {
    let db = conn.lock().unwrap();

    let result: Result<String, _> = db.query_row(
        "SELECT value FROM settings WHERE key = 'server_secret'",
        [],
        |row| row.get(0),
    );

    match result {
        Ok(secret) => Ok(secret),
        Err(_) => {
            let new_secret = key::generate();
            db.execute(
                "INSERT INTO settings (key, value) VALUES ('server_secret', ?1)",
                [&new_secret],
            )
            .map_err(|e| ConfigError::Database(e.to_string()))?;
            Ok(new_secret)
        }
    }
}
