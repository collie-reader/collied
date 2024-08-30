use clap::{Parser, Subcommand};
use config::{AppState, SharedAppState};

mod config;
mod error;
mod serve;

mod adapter {
    pub mod auth;
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
    let cli = Cli::parse();

    match &cli.commands {
        Commands::Serve { port, daemon } => {
            println!(
                "Starting server on {} in {} mode...",
                port,
                if *daemon { "daemon" } else { "foreground" }
            );

            serve::serve(SharedAppState::new(), &format!("0.0.0.0:{}", port)).await;
        }
        Commands::Key(key) => match &key.commands {
            KeyCommands::New { description } => {
                println!("Generating new keys...");
                let (access_key, secret_key) =
                    collie::auth::key::create(AppState::new().conn, description).unwrap();
                println!("Register the following keys with your client. DO NOT share the secret key with anyone.");
                println!("Access key: {}", access_key);
                println!("Secret key: {}", secret_key);
            }
        },
    }
}
