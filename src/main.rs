use std::{fs::OpenOptions, path::PathBuf, sync::Arc};

use clap::{Parser, Subcommand};
use config::Context;
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
    /// Path to the configuration file
    #[arg(short, long)]
    config: Option<String>,

    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value_t = 33003)]
        port: u16,

        /// Run in daemon mode
        #[arg(short, long, default_value_t = false)]
        daemon: bool,
    },
    /// Manage keys for authorization
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
        /// Human-readable description of the key
        #[arg(long)]
        description: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    let config_path = cli.config.as_ref().map(PathBuf::from);

    match &cli.commands {
        Commands::Serve { port, daemon } => {
            println!(
                "Starting server on {} in {} mode...",
                port,
                if *daemon { "daemon" } else { "foreground" }
            );

            let ctx = Arc::new(Context::new(config_path.as_deref()));

            if *daemon {
                let daemonize = Daemonize::new().pid_file(&ctx.config.daemon.pid_file);
                let daemonize = match &ctx.config.daemon.error_log {
                    Some(error_log) => daemonize.stderr(
                        OpenOptions::new()
                            .create(true)
                            .append(true)
                            .read(true)
                            .open(error_log)
                            .unwrap(),
                    ),
                    None => daemonize,
                };

                daemonize.start().unwrap();
            }

            serve::serve(ctx, &format!("0.0.0.0:{}", port));
        }
        Commands::Key(key) => match &key.commands {
            KeyCommands::New { description } => {
                println!("Generating new keys...");
                let (access_key, secret_key) = collie::auth::key::create(
                    Context::new(config_path.as_deref()).conn,
                    description.as_deref(),
                )
                .unwrap();
                println!("Register the following keys with your client. DO NOT share the secret key with anyone.");
                println!("Access key: {}", access_key);
                println!("Secret key: {}", secret_key);
            }
        },
    }
}
