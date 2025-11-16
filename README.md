# Stool

A personal CLI tool for Mac/Linux terminal tasks written in Rust.

**Tech Stack:** Rust 2024 Edition

## Quick Start

```bash
git clone https://github.com/seokjin0414/stool.git
cd stool
cp config.yaml.example config.yaml
vim config.yaml
./install.sh                        # Or: ./install.sh /path/to/config.yaml
stool --help
```

## Features

### SSH Connection
- Interactive server selection menu
- Manual IP input option
- Cancel option (silent exit)
- Multiple authentication methods:
  - PEM key authentication
  - Password authentication (with expect)
    - Auto-accepts host key fingerprint
    - Auto-enters password
  - Password prompt (if not in config)
    - Masked password input
    - Optional: leave empty for default SSH authentication
  - Default SSH key (ssh-agent, ~/.ssh/config)
- Server configuration embedded at build time
- External config file support

### System Update
- Update Homebrew packages
- Update Rust toolchain
- Selective or batch updates

### Filesystem Operations
- Find files by pattern (exact, glob, or partial match)
- Count files and directories

### File Transfer
- SCP-based file transfer
- Upload/Download support
- Server selection from config or manual IP input
- Cancel option (silent exit)
- Same authentication methods as SSH (including password prompt)
- Default paths: Upload(~/), Download(~/Downloads/)
- External config file support
- **Tab completion for local file paths**
- **Empty input support for default paths**
- **Masked password input when not in config**

### Docker Operations
- **Build**: Build Docker images with standardized options
  - Platform: `linux/arm64`
  - Options: `--provenance=false --sbom=false`
  - Tag: `{image}:latest`
  - Image selection from config or manual input
- **Push**: Build, tag, and push to AWS ECR
  - Automatic version management with ECR integration
  - Version types: major, middle, minor (minor default)
  - Initial version: `0.1.0`
  - Always pushes both `latest` and version tags
  - Auto-increments version based on current ECR tags

### AWS CLI Wrapper
- Interactive AWS credential configuration
- Wraps `aws configure` command
- Command aliases: `configure`, `conf`
- ECR login with interactive registry selection
- Supports YAML config and manual input
- Automatic `aws ecr get-login-password` + `docker login` pipeline

### Shell Completion
- Auto-completion for Zsh, Bash, Fish, PowerShell
- Automatically installed with install.sh

## Installation

### Prerequisites
- Rust toolchain (automatically installed by install.sh)
- macOS or Linux

### Quick Install

```bash
# 1. Clone repository
git clone https://github.com/seokjin0414/stool.git
cd stool

# 2. Create config.yaml
cp config.yaml.example config.yaml
vim config.yaml  # Edit with your server information

# 3. Run installation script
./install.sh
```

The script will:
- Check/install Rust
- Build release binary with embedded config.yaml
- Install to `~/Library/Stool/stool`
- Create symlink at `/usr/local/bin/stool`
- Auto-detect and install zsh completion (if writable)

**Alternative: Specify config path**
```bash
./install.sh /path/to/my-config.yaml
```

**Note:** If zsh completion installation fails due to permissions, install manually:
```bash
stool completion zsh | sudo tee /opt/homebrew/share/zsh/site-functions/_stool
source ~/.zshrc
```

### Manual Installation

```bash
# 1. Create config.yaml (required)
cp config.yaml.example config.yaml
vim config.yaml

# 2. Build release
cargo build --release

# 3. Install binary
mkdir -p ~/Library/Stool
cp target/release/stool ~/Library/Stool/

# 4. Create symlink
sudo ln -sf ~/Library/Stool/stool /usr/local/bin/stool

# 5. Install shell completion (optional)
stool completion zsh | sudo tee /usr/local/share/zsh/site-functions/_stool
source ~/.zshrc
```

## Usage

### Help and Version
```bash
stool --help
stool --version
```

### SSH Connection
```bash
stool ssh                          # Use embedded config.yaml
stool -s                           # Short flag
stool ssh --config servers.yaml    # Use external config file
```

### System Update
```bash
stool update           # Update both brew and rustup
stool -u               # Short flag
stool -u --brew        # Update Homebrew only
stool -u --rustup      # Update Rust toolchain only
```

### Filesystem Operations
```bash
stool -f find "*.rs"              # Find with glob pattern
stool -f find "main.rs"           # Find exact filename
stool -f find "main"              # Find with partial match
stool -f find "*.toml" -p ./src   # Find with custom path
stool -f count                    # Count in current directory
stool -f count ./src              # Count in specific path
```

### File Transfer
```bash
stool transfer                         # Use embedded config.yaml
stool -t                               # Short flag
stool transfer --config servers.yaml   # Use external config file
```

**Features:**
- Upload:
  - Local file path supports tab completion (required)
  - Remote path accepts empty input for default (~/)
- Download:
  - Remote file path is required
  - Local destination path supports tab completion, empty input for default (~/Downloads/)

### Docker Operations
```bash
stool docker build                     # Build Docker image only
stool -d build                         # Short flag
stool -d build -c config.yaml          # Use external config file

stool docker push                      # Build + tag + push to ECR
stool -d push                          # Short flag
stool -d push -c config.yaml           # Use external config file
```

**Workflow:**
1. Select ECR registry from config
2. Select or input Docker image name
3. (For push) Build image with `--platform linux/arm64 --provenance=false --sbom=false`
4. (For push) Select version type:
   - major: 0.1.0 → 1.0.0
   - middle: 0.1.0 → 0.2.0
   - minor: 0.1.0 → 0.1.1 (default)
5. (For push) Tag and push both `latest` and version tags

### AWS CLI
```bash
stool aws configure             # Configure AWS credentials
stool -a configure              # Short flag
stool -a conf                   # Alias

stool aws ecr                   # ECR login (embedded config.yaml)
stool -a ecr                    # Short flag
stool -a ecr -c servers.yaml    # ECR login (external config file)
```

### Shell Completion
```bash
stool completion zsh              # Generate zsh completion
stool completion bash             # Generate bash completion
stool completion fish             # Generate fish completion
stool completion powershell       # Generate powershell completion
```

## Configuration

### config.yaml Format
```yaml
servers:
  - name: "Production Server"
    ip: "192.168.1.100"
    user: "admin"
    password: "your-password"  # Optional: password authentication

  - name: "Development Server"
    ip: "192.168.1.101"
    user: "dev"
    key_path: "~/.ssh/id_rsa"  # Optional: PEM key authentication

  - name: "Staging Server"
    ip: "10.0.0.50"
    user: "deploy"
    # No password or key_path - uses default SSH authentication

ecr_registries:
  - name: "Production ECR"
    account_id: "123456789012"    # 12-digit AWS account ID
    region: "ap-northeast-2"      # AWS region
    images:                       # Optional: Docker image names
      - "my-app"
      - "my-service"

  - name: "Dev ECR"
    account_id: "987654321098"
    region: "us-east-1"
    images:
      - "dev-app"
```

### Authentication Priority
1. `key_path` - PEM key authentication
2. `password` - Password with expect script
3. If neither exists - Password prompt with masked input
   - Enter password: Uses expect script for authentication
   - Leave empty: Uses default SSH authentication (ssh-agent, ~/.ssh/config)

### Updating Configuration
```bash
# Edit config and rebuild
vim config.yaml
cargo build --release
cp target/release/stool ~/Library/Stool/

# Or use external config without rebuild
stool ssh --config /path/to/other-config.yaml
```

## Shell Completion Setup

### Zsh (Oh My Zsh)
```bash
# Automatically installed by install.sh (if writable)
# Or manually install to Homebrew path:
stool completion zsh | sudo tee /opt/homebrew/share/zsh/site-functions/_stool
source ~/.zshrc

# Or system path:
stool completion zsh | sudo tee /usr/local/share/zsh/site-functions/_stool
source ~/.zshrc
```

### Bash
```bash
stool completion bash | sudo tee /etc/bash_completion.d/stool
source ~/.bashrc
```

### Fish
```bash
stool completion fish > ~/.config/fish/completions/stool.fish
```

## Project Structure

```
stool/
├── stool-cli/         # Binary crate (CLI interface)
├── stool-core/        # Core types, config, and error handling
│   ├── config.rs      # YAML config loading (Server, EcrRegistry)
│   └── error.rs       # Unified error types and Result alias
├── stool-modules/     # Feature modules (ssh, update, filesystem, transfer, docker, aws)
│   ├── ssh.rs         # SSH connection with server selection
│   ├── update.rs      # System updates (brew, rustup)
│   ├── filesystem.rs  # File search and count operations
│   ├── transfer.rs    # SCP file transfer (upload/download)
│   ├── docker.rs      # Docker operations (build, ECR push with version management)
│   └── aws.rs         # AWS CLI wrapper (configure, ECR login)
└── stool-utils/       # Shared utilities
    ├── interactive.rs # Server selection, text/password/path input (masked, tab completion)
    └── command.rs     # SSH/SCP/command execution with expect -c
```

**Architecture Highlights:**
- Modular workspace structure with clear separation of concerns
- Unified error handling across all modules (25 error types)
- Shared utilities eliminate code duplication (91 lines reduced)
- Optimized for binary size and performance (LTO, strip, single codegen)
- Comprehensive documentation (25 public functions documented)
- Named constants throughout (no magic strings)

## Security Notes

- `config.yaml` is embedded into the binary at build time
- Binary contains server information and credentials
- `config.yaml` is gitignored by default
- Keep built binaries secure
- Use external config files for sensitive environments
- **Password Security:**
  - **expect script method:** Uses `expect -c` to pass scripts as command-line arguments
    - Standard and stable approach for expect automation
    - Passwords embedded in script string but cleared after process termination
    - Alternative stdin method causes conflicts with interactive mode
  - **Interactive password prompt:** Masked input using dialoguer::Password
  - **ECR passwords:** Passed via stdin to docker login (--password-stdin)

## Development

### Code Quality Standards

**Error Handling:**
- All errors use unified `StoolErrorType` enum (25 variants)
- All error messages in English for consistency
- `unwrap()` is completely prohibited; use `?` operator or `map_err()`
- Error messages include contextual information (user@ip, paths, etc.)
- Error chaining with `with_message()` and `with_source()`
- Explicit handling of recoverable errors (e.g., directory read failures)

**Code Organization:**
- Common logic extracted to utility functions
- No code duplication (91 lines eliminated via helpers)
- Named constants for all hardcoded values
- Single responsibility per function

**Documentation:**
- Module-level docs (`//!`) on every module file
- Doc comments (`///`) on all public functions
- Arguments, Returns, and Errors sections included
- 25 public functions fully documented

**Message Formatting:**
- Success: "Action completed successfully" pattern
- Progress: No ellipsis, clear statements
- Errors: Detailed context with relevant information

**Performance:**
- LTO (Link Time Optimization) enabled
- Single codegen unit compilation
- Symbol stripping for smaller binaries
- Panic abort mode

### Check Code
```bash
cargo check && cargo fmt && cargo clippy -- -D warnings
```

### Build
```bash
cargo build --release
```

### Test
```bash
# Test help
./target/release/stool --help

# Test filesystem operations
./target/release/stool -f find "*.toml"
./target/release/stool -f count .
```

## License

MIT License

Copyright (c) 2024 seokjin0414

## Author

seokjin0414 <sars21@hanmail.net>
