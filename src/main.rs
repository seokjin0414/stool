mod config;
mod error;
mod modules;
mod utils;

use clap::{Parser, Subcommand};
use error::Result;

#[derive(Parser)]
#[command(name = "stool")]
#[command(about = "seokjin's CLI tool for Mac/Linux terminal tasks", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(short_flag = 's', about = "SSH connection")]
    Ssh {
        #[arg(short, long, default_value = "servers.yaml")]
        config: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ssh { config } => {
            let cfg = config::Config::load(&config)?;
            modules::ssh::connect(&cfg.servers)?;
        }
    }

    Ok(())
}
