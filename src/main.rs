use clap::{Parser, Subcommand};

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
    },
    /// Manage tokens for client authentication
    #[clap(name = "token")]
    Token(Token),
}

#[derive(Parser)]
pub struct Token {
    #[command(subcommand)]
    pub commands: TokenCommands,
}

#[derive(Subcommand)]
pub enum TokenCommands {
    /// Generate a new token
    New,
    /// Expire a token by its id
    Expire {
        #[arg(long)]
        id: u32,
    },
    /// List all tokens
    List,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.commands {
        Commands::Serve { port } => {
            println!("Starting server on {}...", port);
        }
        Commands::Token(token) => match &token.commands {
            TokenCommands::New => {
                println!("new");
            }
            TokenCommands::Expire { id } => {
                println!("expire {}", id);
            }
            TokenCommands::List => {
                println!("list");
            }
        },
    }
}
