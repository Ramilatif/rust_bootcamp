use clap::{Parser, Subcommand};

mod dh;
mod cipher;
mod net;
mod chat;

/// Stream cipher chat with Diffie-Hellman key generation
#[derive(Parser, Debug)]
#[command(
    name = "streamchat",
    about = "Stream cipher chat with Diffie-Hellman key generation"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start server
    Server {
        /// Port to listen on
        port: u16,
    },
    /// Connect to server
    Client {
        /// Server address (ip:port)
        addr: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Server { port } => {
            println!("[SERVER] Starting on port {port}...");
            net::run_server(port)?;
        }
        Commands::Client { addr } => {
            println!("[CLIENT] Connecting to {addr}...");
            net::run_client(&addr)?;
        }
    }

    Ok(())
}

