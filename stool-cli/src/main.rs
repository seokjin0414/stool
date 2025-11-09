use clap::{ArgAction, CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use std::io;
use stool_core::config::Config;
use stool_core::error::Result;
use stool_modules::{aws, filesystem, ssh, transfer, update};

#[derive(Parser)]
#[command(name = "stool")]
#[command(version)]
#[command(about = "seokjin's CLI tool for Mac/Linux terminal tasks", long_about = None)]
#[command(arg_required_else_help = true)]
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
        #[arg(
            short,
            long,
            help = "External config file (default: embedded config.yaml)"
        )]
        config: Option<String>,
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
    #[command(short_flag = 't', about = "File transfer (scp)")]
    Transfer {
        #[arg(
            short,
            long,
            help = "External config file (default: embedded config.yaml)"
        )]
        config: Option<String>,
    },
    #[command(short_flag = 'a', about = "AWS CLI wrapper")]
    Aws {
        #[command(subcommand)]
        command: AwsCommands,
    },
    #[command(about = "Generate shell completion script")]
    Completion {
        #[arg(value_enum, help = "Shell type (bash, zsh, fish, powershell)")]
        shell: Shell,
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

#[derive(Subcommand)]
enum AwsCommands {
    #[command(
        alias = "configure",
        alias = "conf",
        about = "Configure AWS credentials (aws configure)"
    )]
    Configure,
    #[command(about = "Login to AWS ECR registry")]
    Ecr {
        #[arg(
            short,
            long,
            help = "External config file (default: embedded config.yaml)"
        )]
        config: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Ssh { config }) => {
            let cfg = if let Some(path) = config {
                Config::load(&path)?
            } else {
                Config::load_embedded()?
            };
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
        Some(Commands::Transfer { config }) => {
            let cfg = if let Some(path) = config {
                Config::load(&path)?
            } else {
                Config::load_embedded()?
            };
            transfer::transfer(&cfg.servers)?;
        }
        Some(Commands::Aws { command }) => match command {
            AwsCommands::Configure => {
                aws::configure()?;
            }
            AwsCommands::Ecr { config } => {
                let cfg = if let Some(path) = config {
                    Config::load(&path)?
                } else {
                    Config::load_embedded()?
                };
                aws::ecr_login(&cfg.ecr_registries)?;
            }
        },
        Some(Commands::Completion { shell }) => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "stool", &mut io::stdout());
        }
        None => unreachable!("arg_required_else_help ensures a subcommand is provided"),
    }

    Ok(())
}
