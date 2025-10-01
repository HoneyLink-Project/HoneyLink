# WSL環境での開発に関する注意事項

## パフォーマンス最適化

### ファイルシステムの配置

**重要**: プロジェクトは必ずLinuxファイルシステム（`/home/`）に配置してください。

```bash
# ✅ 推奨（高速）
~/HoneyLink/

# ❌ 避ける（10倍以上遅い）
/mnt/c/Users/Aqua/Programming/HoneyLink/
```

### ビルド時間の比較

| 環境 | `cargo build --workspace` | `cargo test --workspace` |
|------|---------------------------|--------------------------|
| WSL2 (Linux FS) | 約30秒 | 約15秒 |
| WSL2 (Windows FS) | 約5分 | 約2分 |
| Windows (MSVC) | ❌ リンカーエラー | ❌ |

## VS Code統合

### 推奨拡張機能

WSL環境内で以下の拡張機能をインストール:

1. **rust-lang.rust-analyzer** - Rust LSP
2. **vadimcn.vscode-lldb** - デバッガ
3. **dbaeumer.vscode-eslint** - TypeScript/JavaScript リンター
4. **esbenp.prettier-vscode** - コードフォーマッター

### ターミナル設定

WSL内のbashをデフォルトターミナルとして使用することを推奨:

```json
{
  "terminal.integrated.defaultProfile.linux": "bash"
}
```

## Git設定

### 改行コードの統一

```bash
git config --global core.autocrlf input
git config --global core.eol lf
```

### Windowsとの共有リポジトリ

Windows側とWSL側で同じリポジトリを共有しないでください。  
ファイルパーミッションや改行コードの問題が発生します。

## トラブルシューティング

### cargo buildが遅い

**原因**: `/mnt/c/` のWindowsファイルシステムを使用している  
**解決策**: プロジェクトを `~/` に移動

### rust-analyzerが動作しない

```bash
rustup component remove rust-analyzer
rustup component add rust-analyzer
code .  # VS Codeを再起動
```

### Node.jsのパーミッションエラー

```bash
mkdir -p ~/.npm-global
npm config set prefix '~/.npm-global'
echo 'export PATH=~/.npm-global/bin:$PATH' >> ~/.bashrc
source ~/.bashrc
```

## メモリ使用量の制限

大規模プロジェクトの場合、WSL2のメモリ使用量を制限することを推奨:

```powershell
# Windows側で %UserProfile%\.wslconfig を作成
[wsl2]
memory=8GB
processors=4
swap=4GB
```

## バックアップ

WSL環境のバックアップ方法:

```powershell
# Windows PowerShell でエクスポート
wsl --export Ubuntu-22.04 C:\Backups\ubuntu-honeylink.tar

# インポート（復元）
wsl --import Ubuntu-22.04-Restore C:\WSL\Ubuntu-Restore C:\Backups\ubuntu-honeylink.tar
```

## 参考リンク

- [WSL Best Practices](https://docs.microsoft.com/en-us/windows/wsl/setup/environment)
- [VS Code Remote Development](https://code.visualstudio.com/docs/remote/wsl)
