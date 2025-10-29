#!/bin/bash

set -e

echo "🚀 Stool Installation Script"
echo "============================"

# config.yaml 경로 설정
CONFIG_PATH="${1:-config.yaml}"

# config.yaml 체크
echo "📋 Checking config.yaml..."
if [ ! -f "$CONFIG_PATH" ]; then
    echo "❌ config.yaml not found at: $CONFIG_PATH"
    echo ""
    echo "Please create config.yaml before installation:"
    echo "  cp config.yaml.example config.yaml"
    echo "  vim config.yaml"
    echo ""
    echo "Or specify config path:"
    echo "  ./install.sh /path/to/config.yaml"
    exit 1
fi
echo "✅ config.yaml found at: $CONFIG_PATH"

# config.yaml 복사 (프로젝트 루트에)
if [ "$CONFIG_PATH" != "config.yaml" ]; then
    echo "📋 Copying config to project root..."
    cp "$CONFIG_PATH" config.yaml
    echo "✅ Config copied to config.yaml"
fi

# Rust 설치 체크
echo "📋 Checking Rust installation..."
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    echo "✅ Rust is already installed: $RUST_VERSION"
    echo "🔄 Updating Rust..."
    rustup update
else
    echo "❌ Rust not found. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo "✅ Rust installed successfully"
fi

# 설치 재확인
echo "🔍 Verifying Rust installation..."
if command -v rustc &> /dev/null && command -v cargo &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    CARGO_VERSION=$(cargo --version)
    echo "✅ Rust verification passed:"
    echo "   - $RUST_VERSION"
    echo "   - $CARGO_VERSION"
else
    echo "❌ Rust verification failed. Please install Rust manually."
    exit 1
fi

# Release 빌드
echo "🔨 Building Stool (release mode)..."
cargo build --release

# 빌드 성공 체크
if [ -f "target/release/stool" ]; then
    echo "✅ Build successful"
else
    echo "❌ Build failed - executable not found"
    exit 1
fi

# Library 폴더에 설치
echo "📦 Installing to ~/Library/Stool..."
STOOL_DIR="$HOME/Library/Stool"
mkdir -p "$STOOL_DIR"
cp target/release/stool "$STOOL_DIR/stool"
chmod +x "$STOOL_DIR/stool"
echo "✅ Installed to $STOOL_DIR/stool"

# 커맨드 등록 (심볼릭 링크)
echo "🔗 Creating symbolic link..."
SYMLINK_PATH="/usr/local/bin/stool"

if [ -w "/usr/local/bin" ]; then
    ln -sf "$STOOL_DIR/stool" "$SYMLINK_PATH"
    echo "✅ Command registered to $SYMLINK_PATH"
else
    echo "⚠️  Permission required. Running with sudo..."
    sudo ln -sf "$STOOL_DIR/stool" "$SYMLINK_PATH"
    echo "✅ Command registered to $SYMLINK_PATH"
fi

# Zsh completion 설치
echo "📝 Installing zsh completion..."

# Find writable zsh completion directory
COMPLETION_DIR=""
for dir in /opt/homebrew/share/zsh/site-functions /usr/local/share/zsh/site-functions /usr/share/zsh/site-functions; do
    if [ -d "$dir" ] && [ -w "$dir" ]; then
        COMPLETION_DIR="$dir"
        break
    fi
done

if [ -n "$COMPLETION_DIR" ]; then
    "$STOOL_DIR/stool" completion zsh > "$COMPLETION_DIR/_stool" 2>/dev/null
    if [ $? -eq 0 ]; then
        echo "✅ Zsh completion installed to $COMPLETION_DIR/_stool"
        echo "💡 Restart your shell or run: source ~/.zshrc"
    else
        echo "⚠️  Failed to generate completion. Skipping..."
    fi
else
    echo "⚠️  No writable zsh completion directory found. Skipping..."
    echo "💡 Manually install: stool completion zsh > ~/.zsh/completions/_stool"
fi

# 설치 확인
if command -v stool &> /dev/null; then
    STOOL_VERSION=$(stool --version 2>&1 || echo "version check failed")
    echo "🎉 Installation completed successfully!"
    echo "📋 Installed: $STOOL_VERSION"
    echo ""
    echo "💡 Usage:"
    echo "   stool --help       # Show help"
    echo "   stool -s           # SSH connection"
    echo "   stool -u           # System update"
    echo "   stool -f find      # Find files"
    echo "   stool -a conf      # AWS configure"
else
    echo "❌ Installation verification failed"
    exit 1
fi
