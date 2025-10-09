use clap::{Parser, Subcommand};
use stool_core::config::Config;
use stool_core::error::Result;
use stool_modules::{ssh, update};

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
    #[command(short_flag = 'u', about = "System updates (brew, rustup)")]
    Update {
        #[arg(long, help = "Update Homebrew only")]
        brew: bool,
        #[arg(long, help = "Update Rust toolchain only")]
        rustup: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ssh { config } => {
            let cfg = Config::load(&config)?;
            ssh::connect(&cfg.servers)?;
        }
        Commands::Update { brew, rustup } => match (brew, rustup) {
            (true, false) => update::update_brew()?,
            (false, true) => update::update_rustup()?,
            _ => update::update_all()?,
        },
    }

    Ok(())
}
