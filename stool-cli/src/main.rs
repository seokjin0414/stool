use clap::{ArgAction, CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};
use std::io;
use stool_core::config::Config;
use stool_core::error::Result;
use stool_modules::{aws, docker, filesystem, ssh, transfer, update};

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
    #[command(
        short_flag = 's',
        about = "SSH connection",
        long_about = "Connect to remote servers via SSH with multiple authentication methods\n\nFeatures:\n  - Server selection from config or manual input\n  - PEM key authentication\n  - Password authentication with expect\n  - Password prompt with masked input\n  - Default SSH authentication (ssh-agent, ~/.ssh/config)"
    )]
    Ssh {
        #[arg(
            short,
            long,
            help = "External config file (default: embedded config.yaml)"
        )]
        config: Option<String>,
    },
    #[command(
        short_flag = 'u',
        about = "System updates (brew, rustup)",
        long_about = "Update system packages and toolchains\n\nOptions:\n  --brew   - Update Homebrew packages only\n  --rustup - Update Rust toolchain only\n  (no flags) - Update both brew and rustup"
    )]
    Update {
        #[arg(long, help = "Update Homebrew only")]
        brew: bool,
        #[arg(long, help = "Update Rust toolchain only")]
        rustup: bool,
    },
    #[command(
        short_flag = 'f',
        about = "Filesystem operations",
        long_about = "File search and directory operations\n\nCommands:\n  find  - Find files by pattern (exact, glob, or partial match)\n  count - Count files and directories in a path"
    )]
    Filesystem {
        #[command(subcommand)]
        command: FilesystemCommands,
    },
    #[command(
        short_flag = 't',
        about = "File transfer (scp)",
        long_about = "Transfer files between local and remote systems via SCP\n\nFeatures:\n  - Upload/Download support\n  - Server selection from config or manual input\n  - Tab completion for local file paths\n  - Default paths: Upload(~/), Download(~/Downloads/)\n  - Same authentication methods as SSH"
    )]
    Transfer {
        #[arg(
            short,
            long,
            help = "External config file (default: embedded config.yaml)"
        )]
        config: Option<String>,
    },
    #[command(
        short_flag = 'd',
        about = "Docker operations",
        long_about = "Docker image build and ECR deployment operations\n\nCommands:\n  build - Build Docker image with standardized options\n  push  - Build, tag, and push to AWS ECR with version management"
    )]
    Docker {
        #[command(subcommand)]
        command: DockerCommands,
    },
    #[command(
        short_flag = 'a',
        about = "AWS CLI wrapper",
        long_about = "AWS CLI operations and ECR authentication\n\nCommands:\n  configure - Interactive AWS credential configuration\n  ecr       - Login to AWS ECR registry"
    )]
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
enum DockerCommands {
    #[command(
        about = "Build Docker image",
        long_about = "Build Docker image with platform options (linux/arm64, --provenance=false, --sbom=false)\nImage name can be selected from config or manually entered"
    )]
    Build {
        #[arg(
            short,
            long,
            help = "External config file (default: embedded config.yaml)"
        )]
        config: Option<String>,
    },
    #[command(
        about = "Build and push Docker image to ECR",
        long_about = "Build, tag, and push Docker image to AWS ECR with automatic version management\n\nWorkflow:\n1. Select ECR registry\n2. Select or input image name\n3. Build with standard options\n4. Select version type (major/middle/minor)\n5. Tag and push both 'latest' and version tags"
    )]
    Push {
        #[arg(
            short,
            long,
            help = "External config file (default: embedded config.yaml)"
        )]
        config: Option<String>,
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
        Some(Commands::Docker { command }) => match command {
            DockerCommands::Build { config } => {
                let cfg = if let Some(path) = config {
                    Config::load(&path)?
                } else {
                    Config::load_embedded()?
                };
                docker::build_only(&cfg.ecr_registries)?;
            }
            DockerCommands::Push { config } => {
                let cfg = if let Some(path) = config {
                    Config::load(&path)?
                } else {
                    Config::load_embedded()?
                };
                docker::push_to_ecr(&cfg.ecr_registries)?;
            }
        },
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
