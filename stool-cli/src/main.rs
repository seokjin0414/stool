use clap::{Parser, Subcommand};
use stool_core::config::Config;
use stool_core::error::Result;
use stool_modules::ssh;

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
            let cfg = Config::load(&config)?;
            ssh::connect(&cfg.servers)?;
        }
    }

    Ok(())
}
