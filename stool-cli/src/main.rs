use clap::{ArgAction, Parser, Subcommand};
use stool_core::config::Config;
use stool_core::error::Result;
use stool_modules::{filesystem, ssh, update};

#[derive(Parser)]
#[command(name = "stool")]
#[command(version)]
#[command(about = "seokjin's CLI tool for Mac/Linux terminal tasks", long_about = None)]
struct Cli {
    #[arg(short = 'v', short_alias = 'V', long, action = ArgAction::Version)]
    version: Option<bool>,

    #[arg(short = 'h', short_alias = 'H', long, action = ArgAction::Help)]
    help: Option<bool>,

    #[command(subcommand)]
    command: Option<Commands>,
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
    #[command(short_flag = 'f', about = "Filesystem operations")]
    Filesystem {
        #[command(subcommand)]
        command: FilesystemCommands,
    },
}

#[derive(Subcommand)]
enum FilesystemCommands {
    #[command(about = "Find files by pattern")]
    Find {
        #[arg(help = "Search pattern (exact, glob, or partial)")]
        pattern: String,
        #[arg(short, long, help = "Search path (default: current directory)")]
        path: Option<String>,
    },
    #[command(about = "Count files and directories")]
    Count {
        #[arg(help = "Target path (default: current directory)")]
        path: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Ssh { config }) => {
            let cfg = Config::load(&config)?;
            ssh::connect(&cfg.servers)?;
        }
        Some(Commands::Update { brew, rustup }) => match (brew, rustup) {
            (true, false) => update::update_brew()?,
            (false, true) => update::update_rustup()?,
            _ => update::update_all()?,
        },
        Some(Commands::Filesystem { command }) => match command {
            FilesystemCommands::Find { pattern, path } => {
                filesystem::find(&pattern, path.as_deref())?;
            }
            FilesystemCommands::Count { path } => {
                filesystem::count(path.as_deref())?;
            }
        },
        None => {
            // Version flag is already handled by clap
            Cli::parse_from(["stool", "--help"]);
        }
    }

    Ok(())
}
