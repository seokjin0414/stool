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

if echo "$PATH" | tr ':' '\n' | grep -qxF "$HOME/.local/bin"; then
    # $HOME/.local/bin이 이미 PATH에 등록되어 있으면 sudo 없이 여기에 설치
    mkdir -p "$HOME/.local/bin"
    SYMLINK_PATH="$HOME/.local/bin/stool"
    ln -sf "$STOOL_DIR/stool" "$SYMLINK_PATH"
    echo "✅ Command registered to $SYMLINK_PATH"
else
    # PATH에 없으면 /usr/local/bin으로 폴백
    BIN_DIR="/usr/local/bin"
    SYMLINK_PATH="$BIN_DIR/stool"

    if [ -d "$BIN_DIR" ] && [ -w "$BIN_DIR" ] && [ -x "$BIN_DIR" ]; then
        ln -sf "$STOOL_DIR/stool" "$SYMLINK_PATH"
        echo "✅ Command registered to $SYMLINK_PATH"
    else
        echo "⚠️  Permission required. Running with sudo..."
        sudo mkdir -p "$BIN_DIR"
        # 디렉토리 권한이 0700(root 전용)이면 심볼릭 링크가 생성되어도
        # 일반 사용자는 디렉토리를 통과(traverse)할 수 없어 command -v stool이
        # 실패하므로, 755 권한을 보장해준다
        sudo chmod 755 "$BIN_DIR"
        sudo ln -sf "$STOOL_DIR/stool" "$SYMLINK_PATH"
        echo "✅ Command registered to $SYMLINK_PATH"
    fi
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
hash -r 2>/dev/null || true
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
    echo ""
    echo "🔍 문제 해결 방법:"
    echo "   1) 설치 경로가 \$PATH에 없을 수 있습니다."
    echo "      -> 새 터미널을 열거나 'hash -r' 실행 후 다시 확인해보세요."
    echo "   2) 설치 경로에 접근 권한이 없을 수 있습니다."
    echo "      -> ls -ld \"$(dirname "$SYMLINK_PATH")\" 로 권한을 확인해보세요."
    echo "      -> drwx------ 처럼 표시된다면 다음 명령으로 권한을 수정하세요:"
    echo "         sudo chmod 755 \"$(dirname "$SYMLINK_PATH")\""
    exit 1
fi
