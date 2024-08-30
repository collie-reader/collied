use std::fs::OpenOptions;

use clap::{Parser, Subcommand};
use config::{AppState, SharedAppState};
use daemonize::Daemonize;

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

fn main() {
    let cli = Cli::parse();

    match &cli.commands {
        Commands::Serve { port, daemon } => {
            println!(
                "Starting server on {} in {} mode...",
                port,
                if *daemon { "daemon" } else { "foreground" }
            );

            let app_state = SharedAppState::new();

            if *daemon {
                let daemonize = Daemonize::new().pid_file(&app_state.config.daemon.pid_file);
                let daemonize = match &app_state.config.daemon.error_log {
                    Some(error_log) => {
                        daemonize.stderr(
                            OpenOptions::new()
                                .create(true)
                                .append(true)
                                .read(true)
                                .open(error_log)
                                .unwrap()
                        )
                    }
                    None => daemonize,
                };

                daemonize.start().unwrap();
            }

            serve::serve(app_state, &format!("0.0.0.0:{}", port));
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
