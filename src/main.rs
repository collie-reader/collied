use std::{path::PathBuf, sync::Mutex};

use clap::{Parser, Subcommand};
use models::database;
use rusqlite::Connection;

mod config;
mod error;

mod core {
    pub mod auth {
        pub mod key;
        pub mod token;
    }
}

mod models {
    pub mod database;
    pub mod key;
}

pub struct DbState {
    db: Mutex<Connection>,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the server on the specified port
    Serve {
        #[arg(short, long, default_value_t = 33003)]
        port: u16,

        #[arg(short, long, default_value_t = false)]
        daemon: bool,
    },
    /// Manage keys for client authentication
    #[clap(name = "key")]
    Key(Key),
}

#[derive(Parser)]
pub struct Key {
    #[command(subcommand)]
    pub commands: KeyCommands,
}

#[derive(Subcommand)]
pub enum KeyCommands {
    /// Generate a new key
    New {
        #[arg(long)]
        description: Option<String>,
    },
    /// Expire a key by its id
    Expire {
        #[arg(long)]
        id: u32,
    },
    /// List all keys
    List,
}

#[tokio::main]
async fn main() {
    let config = read_config();
    let db = open_database_connection(&config);
    let _ = database::migrate(&db);

    let cli = Cli::parse();
    let db_state = DbState { db: Mutex::new(db) };

    match &cli.commands {
        Commands::Serve { port, daemon } => {
            println!(
                "Starting server on {} in {} mode",
                port,
                if *daemon { "daemon" } else { "foreground" }
            );
        }
        Commands::Key(key) => match &key.commands {
            KeyCommands::New { description } => {
                println!("Generating new key...");
                let access_key = core::auth::key::create(&db_state, description).unwrap();
                println!("Access key: {}", access_key);
            }
            KeyCommands::Expire { id } => {
                core::auth::key::expire(&db_state, *id);
            }
            KeyCommands::List => {
                core::auth::key::read_all(&db_state);
            }
        },
    }
}

fn open_database_connection(config: &config::Config) -> Connection {
    models::database::open_connection(&PathBuf::from(config.database.path.clone())).unwrap()
}

fn read_config() -> config::Config {
    config::from(&PathBuf::from("data/config.toml"))
}
