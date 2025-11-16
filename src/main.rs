use std::{
    fs::OpenOptions,
    io::{self, Write},
    path::PathBuf,
    sync::Arc,
};

use clap::{Parser, Subcommand};
use collie::auth;
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
        #[arg(short, long, default_value_t = 3000)]
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

            let ctx = match Context::new(config_path.as_deref()) {
                Ok(ctx) => Arc::new(ctx),
                Err(e) => {
                    eprintln!("Failed to initialize server: {}", e);
                    std::process::exit(1);
                }
            };

            if *daemon {
                let daemonize = Daemonize::new().pid_file(&ctx.config.daemon.pid_file);
                let daemonize = match &ctx.config.daemon.error_log {
                    Some(error_log) => daemonize.stderr(
                        OpenOptions::new()
                            .create(true)
                            .append(true)
                            .read(true)
                            .open(error_log)
                            .unwrap_or_else(|e| {
                                eprintln!("Failed to open error log file: {}", e);
                                std::process::exit(1);
                            }),
                    ),
                    None => daemonize,
                };

                if let Err(e) = daemonize.start() {
                    eprintln!("Failed to daemonize: {}", e);
                    std::process::exit(1);
                }
            }

            serve::serve(ctx, &format!("0.0.0.0:{}", port));
        }
        Commands::Key(key) => match &key.commands {
            KeyCommands::New { description } => {
                println!("Generating new keys...");
                let ctx = match Context::new(config_path.as_deref()) {
                    Ok(ctx) => ctx,
                    Err(e) => {
                        eprintln!("Failed to initialize: {}", e);
                        std::process::exit(1);
                    }
                };
                let (access_key, secret_key) =
                    match auth::service::key::create(ctx.conn, description.as_deref()) {
                        Ok(keys) => keys,
                        Err(e) => {
                            eprintln!("Failed to create keys: {}", e);
                            std::process::exit(1);
                        }
                    };

                println!();
                println!("Register the following keys with your client. DO NOT share the secret key with anyone.");
                println!("Save these keys now. The secret key will NOT be shown again.");
                println!("- Access key: {}", access_key);
                println!("- Secret key: {}", secret_key);
                println!();

                print!("Press Enter to clear the screen...");
                let _ = io::stdout().flush();
                let _ = io::stdin().read_line(&mut String::new());

                print!("\x1B[2J\x1B[1;1H"); // ANSI escape to clear screen
                let _ = io::stdout().flush();
                println!("Key generation complete.");
            }
        },
    }
}
