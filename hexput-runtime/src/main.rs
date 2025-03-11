pub mod error;
pub mod handler;
pub mod messages;
pub mod server;

use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "127.0.0.1")]
    address: String,

    #[arg(short, long, default_value = "9001")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();
    let server_address = format!("{}:{}", args.address, args.port);

    info!(
        "Starting Hexput Runtime WebSocket server on {}",
        server_address
    );

    let config = server::ServerConfig {
        address: server_address,
    };

    server::run_server(config).await?;

    Ok(())
}
