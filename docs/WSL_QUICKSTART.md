# WSL2 クイックスタートガイド

## 最速セットアップ（5分）

### 1. WSL2をインストール（Windows PowerShell - 管理者権限）

```powershell
wsl --install -d Ubuntu-22.04
```

再起動後、Ubuntuが起動してユーザー名・パスワードを設定。

### 2. 自動セットアップスクリプトを実行（WSL内）

```bash
curl -sSL https://raw.githubusercontent.com/HoneyLink-Project/HoneyLink/master/scripts/setup-wsl.sh | bash
```

### 3. プロジェクトをクローン

```bash
cd ~
git clone https://github.com/HoneyLink-Project/HoneyLink.git
cd HoneyLink
```

### 4. ビルド確認

```bash
# Rust
cargo build --workspace

# UI
cd ui && pnpm install && pnpm dev
```

### 5. VS Codeで開く

```bash
code .
```

---

## よくある質問

**Q: Windows側のファイルにアクセスできますか？**  
A: はい。`/mnt/c/Users/...` でアクセスできますが、パフォーマンスが低下するため非推奨です。

**Q: WindowsブラウザからWSLのlocalhostにアクセスできますか？**  
A: はい。`http://localhost:5173` で直接アクセス可能です。

**Q: GitHubの認証はどうすればいいですか？**  
A: SSH鍵を生成するか、GitHub CLIを使用してください:
```bash
# SSH鍵生成
ssh-keygen -t ed25519 -C "your_email@example.com"
cat ~/.ssh/id_ed25519.pub  # GitHubに登録

# または GitHub CLI
curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null
sudo apt update && sudo apt install gh
gh auth login
```

---

詳細は [`docs/WSL_SETUP.md`](./WSL_SETUP.md) を参照してください。
