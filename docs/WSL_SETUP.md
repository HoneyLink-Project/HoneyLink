# WSL2 開発環境セットアップガイド

Windows環境でのMSVCリンカー問題を回避し、Linux環境で快適に開発するためのガイドです。

## 📋 目次

- [WSL2のインストール](#wsl2のインストール)
- [Ubuntu環境のセットアップ](#ubuntu環境のセットアップ)
- [Rust開発環境の構築](#rust開発環境の構築)
- [Node.js環境の構築](#nodejs環境の構築)
- [VS CodeのWSL統合](#vs-codeのwsl統合)
- [プロジェクトのクローン](#プロジェクトのクローン)
- [ビルドと実行](#ビルドと実行)

---

## 🐧 WSL2のインストール

### 1. WSL2を有効化

PowerShellを**管理者権限**で開き、以下を実行:

```powershell
# WSLとVirtual Machine Platformを有効化
wsl --install

# 再起動が必要です
```

再起動後、WSL2がデフォルトバージョンに設定されていることを確認:

```powershell
wsl --set-default-version 2
```

### 2. Ubuntuのインストール

```powershell
# Ubuntu 22.04 LTSをインストール（推奨）
wsl --install -d Ubuntu-22.04
```

初回起動時にユーザー名とパスワードを設定してください。

### 3. WSL2の確認

```powershell
# インストール済みディストリビューションを確認
wsl --list --verbose

# 出力例:
#   NAME            STATE           VERSION
# * Ubuntu-22.04    Running         2
```

---

## ⚙️ Ubuntu環境のセットアップ

WSLターミナルを開き、以下を実行:

```bash
# システムパッケージを最新化
sudo apt update && sudo apt upgrade -y

# 必須ビルドツールをインストール
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    git \
    curl \
    wget \
    ca-certificates \
    gnupg \
    lsb-release
```

---

## 🦀 Rust開発環境の構築

### 1. Rustのインストール

```bash
# Rustup経由でRustをインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# デフォルト設定（1を選択）で進める
# インストール後、環境変数を再読み込み
source "$HOME/.cargo/env"
```

### 2. Rustツールチェーンの設定

```bash
# Rust 1.89.0を指定（プロジェクト要件）
rustup default 1.89.0

# 必要なコンポーネントを追加
rustup component add clippy rustfmt rust-src

# WASMターゲットを追加
rustup target add wasm32-unknown-unknown

# バージョン確認
rustc --version  # 1.89.0 が表示されること
cargo --version
```

### 3. 開発ツールのインストール

```bash
# カバレッジ・監査・ライセンスチェックツール
cargo install cargo-llvm-cov cargo-audit cargo-deny

# WASM開発ツール（オプション）
cargo install wasm-bindgen-cli
```

---

## 📦 Node.js環境の構築

### 1. Node.js (LTS) のインストール

```bash
# Node.js 22.x LTSをインストール
curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
sudo apt install -y nodejs

# バージョン確認
node --version   # v22.15.0 以降
npm --version
```

### 2. pnpmのインストール

```bash
# pnpm（高速パッケージマネージャー）をグローバルインストール
npm install -g pnpm

# バージョン確認
pnpm --version  # 10.x 以降
```

---

## 🔧 VS CodeのWSL統合

### 1. 拡張機能のインストール

VS Code（Windows側）で以下の拡張機能をインストール:

1. **WSL** (ms-vscode-remote.remote-wsl)
   - WSL環境にリモート接続

2. **Remote Development** (ms-vscode-remote.vscode-remote-extensionpack)
   - WSL/SSH/コンテナ統合パック

### 2. WSL内でVS Codeを開く

```bash
# WSLターミナルからプロジェクトディレクトリへ移動
cd ~
git clone https://github.com/HoneyLink-Project/HoneyLink.git
cd HoneyLink

# VS Codeを起動（WSL環境で）
code .
```

初回起動時、VS Code Serverが自動インストールされます。

### 3. WSL内での拡張機能インストール

VS Code（WSL環境）で以下をインストール:

```
Cmd/Ctrl+Shift+P → "Extensions: Show Recommended Extensions"
```

`.vscode/extensions.json` に記載された推奨拡張機能がWSL側にもインストールされます。

---

## 📂 プロジェクトのクローン

### Windowsファイルシステムを避ける

**重要**: パフォーマンスのため、WSLのLinuxファイルシステム（`/home/`）にプロジェクトを配置してください。

```bash
# ❌ 避けるべき（遅い）
cd /mnt/c/Users/Aqua/Programming/HoneyLink

# ✅ 推奨（高速）
cd ~
git clone https://github.com/HoneyLink-Project/HoneyLink.git
cd HoneyLink
```

---

## 🏗️ ビルドと実行

### Rust ワークスペースのビルド

```bash
# 依存関係の解決
cargo fetch

# デバッグビルド
cargo build --workspace

# リリースビルド
cargo build --workspace --release

# テスト実行
cargo test --workspace

# リンター実行
cargo fmt --check
cargo clippy --workspace --all-targets --all-features
```

### UI開発サーバーの起動

```bash
cd ui

# 依存関係インストール
pnpm install

# C/C++依存チェック
node ../scripts/audit-native-deps.js

# 開発サーバー起動
pnpm dev
```

ブラウザで `http://localhost:5173` にアクセス（Windows側のブラウザから）。

---

## 🔄 Pre-commitフックのセットアップ

```bash
# Python3とpipをインストール（未インストールの場合）
sudo apt install -y python3 python3-pip

# pre-commitをインストール
pip3 install pre-commit

# フックを有効化
pre-commit install

# 手動実行テスト
pre-commit run --all-files
```

---

## 🐛 トラブルシューティング

### 問題: `cargo build` が遅い

**解決策**: `/mnt/c/` ではなく、WSLのLinuxファイルシステム（`~/`）を使用してください。

### 問題: VS Codeのrust-analyzerが動作しない

**解決策**:
```bash
# WSL内でrust-analyzerを再インストール
rustup component remove rust-analyzer
rustup component add rust-analyzer
```

### 問題: Node.jsのパーミッションエラー

**解決策**:
```bash
# npmのグローバルディレクトリをユーザー所有に変更
mkdir -p ~/.npm-global
npm config set prefix '~/.npm-global'
echo 'export PATH=~/.npm-global/bin:$PATH' >> ~/.bashrc
source ~/.bashrc
```

### 問題: Windowsとの改行コード問題

**解決策**:
```bash
# Gitの改行コード自動変換を無効化
git config --global core.autocrlf input
git config --global core.eol lf
```

---

## 📊 パフォーマンス比較

| 操作 | Windows (MSVC) | WSL2 (Linux) |
|------|----------------|--------------|
| `cargo build --workspace` | ❌ リンカーエラー | ✅ 約30秒 |
| `cargo test --workspace` | ❌ | ✅ 約15秒 |
| `pnpm install` | 約2分 | 約30秒 |
| ファイルIO | 遅い（/mnt/c/） | 高速（~/） |

---

## 🎯 推奨ワークフロー

```bash
# 1. WSL環境で開発
cd ~/HoneyLink

# 2. VS Codeで編集
code .

# 3. ターミナルでビルド・テスト
cargo build --workspace
cargo test --workspace

# 4. UI開発
cd ui && pnpm dev

# 5. Windowsブラウザでプレビュー
# http://localhost:5173
```

---

## 🔗 参考リンク

- [WSL2公式ドキュメント](https://docs.microsoft.com/ja-jp/windows/wsl/)
- [VS Code WSL拡張機能](https://code.visualstudio.com/docs/remote/wsl)
- [Rust公式インストールガイド](https://www.rust-lang.org/tools/install)

---

**🐧 WSL2環境での快適な開発をお楽しみください！**
