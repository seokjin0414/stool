# Stool

A personal CLI tool for Mac/Linux terminal tasks written in Rust.

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
  - Default SSH key
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
- Same authentication methods as SSH
- Default paths: Upload(~/), Download(~/Downloads/)
- External config file support

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
```

### Authentication Priority
1. `key_path` - PEM key authentication
2. `password` - Password with expect script
3. Default - Standard SSH connection

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
│   ├── config.rs      # YAML config loading (embedded/external)
│   └── error.rs       # Unified error types and Result alias
├── stool-modules/     # Feature modules (ssh, update, filesystem, transfer)
│   ├── ssh.rs         # SSH connection with server selection
│   ├── update.rs      # System updates (brew, rustup)
│   ├── filesystem.rs  # File search and count operations
│   └── transfer.rs    # SCP file transfer (upload/download)
└── stool-utils/       # Shared utilities
    ├── interactive.rs # Common server selection and user input
    └── command.rs     # SSH/SCP/command execution helpers
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

## Development

### Code Quality Standards

**Error Handling:**
- All errors use unified `StoolErrorType` enum (25 variants)
- `unwrap()` is completely prohibited; use `?` operator or `map_err()`
- Error messages include contextual information (user@ip, paths, etc.)
- Error chaining with `with_message()` and `with_source()`

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

Personal project by seokjin0414

## Author

seokjin0414 <sars21@hanmail.net>
