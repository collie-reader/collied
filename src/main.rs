use clap::{Parser, Subcommand};
use collie::{
    auth::model::database::keys_table,
    model::database::{self, feed_table, items_table, Connection},
};
use config::Config;
use std::{path::PathBuf, sync::Mutex};

mod config;
mod error;
mod serve;

mod adapter {
    pub mod feed;
    pub mod item;
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
}

#[tokio::main]
async fn main() {
    let config = read_config();
    let cli = Cli::parse();

    match &cli.commands {
        Commands::Serve { port, daemon } => {
            println!(
                "Starting server on {} in {} mode...",
                port,
                if *daemon { "daemon" } else { "foreground" }
            );

            serve::serve(db_state(&config), &format!("0.0.0.0:{}", port), &config).await;
        }
        Commands::Key(key) => match &key.commands {
            KeyCommands::New { description } => {
                println!("Generating new key...");
                let access_key = collie::auth::key::create(db_state(&config), description).unwrap();
                println!("Access key: {}", access_key);
            }
        },
    }
}

fn read_config() -> Config {
    config::from(&PathBuf::from("data/config.toml"))
}

fn db_state(config: &Config) -> Connection {
    let db = database::open_connection(&PathBuf::from(&config.database.path)).unwrap();

    let _ = database::Migration::new()
        .add_table(feed_table())
        .add_table(items_table())
        .add_table(keys_table())
        .migrate(&db);

    Connection { db: Mutex::new(db) }
}
