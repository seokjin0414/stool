#!/bin/bash

set -e

echo "🚀 Stool Installation Script"
echo "============================"

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
else
    echo "❌ Installation verification failed"
    exit 1
fi
