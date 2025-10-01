#!/bin/bash
# HoneyLink WSL2 自動セットアップスクリプト
#
# 使い方:
#   curl -sSL https://raw.githubusercontent.com/HoneyLink-Project/HoneyLink/master/scripts/setup-wsl.sh | bash

set -e

echo "🍯 HoneyLink™ WSL2 開発環境セットアップを開始します..."

# 色付きログ
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# ========== システムパッケージの更新 ==========
log_info "システムパッケージを更新しています..."
sudo apt update && sudo apt upgrade -y

# ========== 必須ビルドツールのインストール ==========
log_info "必須ビルドツールをインストールしています..."
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    git \
    curl \
    wget \
    ca-certificates \
    gnupg \
    lsb-release \
    python3 \
    python3-pip

# ========== Rustのインストール ==========
if command -v rustc &> /dev/null; then
    log_warn "Rustは既にインストールされています。スキップします。"
else
    log_info "Rustをインストールしています..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Rustツールチェーンの設定
log_info "Rustツールチェーンを設定しています..."
rustup default 1.89.0
rustup component add clippy rustfmt rust-src
rustup target add wasm32-unknown-unknown

# Cargo開発ツールのインストール
log_info "Cargo開発ツールをインストールしています..."
cargo install cargo-llvm-cov cargo-audit cargo-deny || log_warn "一部のツールのインストールに失敗しました"

# ========== Node.jsのインストール ==========
if command -v node &> /dev/null; then
    NODE_VERSION=$(node --version)
    log_warn "Node.js ${NODE_VERSION} は既にインストールされています。"
else
    log_info "Node.js 22.x LTSをインストールしています..."
    curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
    sudo apt install -y nodejs
fi

# ========== pnpmのインストール ==========
if command -v pnpm &> /dev/null; then
    log_warn "pnpmは既にインストールされています。"
else
    log_info "pnpmをインストールしています..."
    npm install -g pnpm
fi

# ========== pre-commitのインストール ==========
log_info "pre-commitをインストールしています..."
pip3 install pre-commit

# ========== Gitの設定 ==========
log_info "Gitの改行コード設定を行っています..."
git config --global core.autocrlf input
git config --global core.eol lf

# ========== バージョン確認 ==========
echo ""
log_info "インストール完了！バージョン情報:"
echo "----------------------------------------"
echo "Rust:     $(rustc --version)"
echo "Cargo:    $(cargo --version)"
echo "Node.js:  $(node --version)"
echo "npm:      $(npm --version)"
echo "pnpm:     $(pnpm --version)"
echo "Python:   $(python3 --version)"
echo "Git:      $(git --version)"
echo "----------------------------------------"

# ========== 次のステップ ==========
echo ""
log_info "セットアップが完了しました！"
echo ""
echo "次のステップ:"
echo "  1. プロジェクトをクローン:"
echo "     cd ~"
echo "     git clone https://github.com/HoneyLink-Project/HoneyLink.git"
echo "     cd HoneyLink"
echo ""
echo "  2. Rustワークスペースをビルド:"
echo "     cargo build --workspace"
echo ""
echo "  3. UI依存関係をインストール:"
echo "     cd ui && pnpm install"
echo ""
echo "  4. VS CodeでWSL環境を開く:"
echo "     code ."
echo ""
echo "  5. pre-commitフックを有効化:"
echo "     pre-commit install"
echo ""
log_info "詳細は docs/WSL_SETUP.md を参照してください。"
echo ""
echo "🍯 Happy Coding with HoneyLink™!"
