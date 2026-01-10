# 🚀 リリース手順

このドキュメントでは、sine-mml の新バージョンをリリースする手順を説明します。

---

## 📌 バージョニング

[セマンティックバージョニング](https://semver.org/lang/ja/) に従います：

| 変更の種類 | バージョン | 例 |
|-----------|----------|-----|
| 後方互換性のない変更 | MAJOR | 1.0.0 → 2.0.0 |
| 後方互換性のある機能追加 | MINOR | 1.0.0 → 1.1.0 |
| バグ修正 | PATCH | 1.0.0 → 1.0.1 |

---

## ✅ リリース前チェックリスト

リリース前に以下を確認してください：

```bash
# 1. 全テストが通ること
cargo test

# 2. Lintが通ること
cargo clippy -- -D warnings

# 3. リリースビルドが成功すること
cargo build --release

# 4. 動作確認
./target/release/sine-mml play "CDE"
./target/release/sine-mml --help
```

---

## 📝 リリース手順

### 1. バージョンを更新

`Cargo.toml` のバージョンを更新：

```toml
[package]
name = "sine-mml"
version = "0.2.0"  # ← 新しいバージョン
```

### 2. 変更をコミット

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to 0.2.0"
```

### 3. タグを作成・プッシュ

```bash
git tag v0.2.0
git push origin master
git push origin v0.2.0
```

### 4. GitHub Release を作成

1. [Releases](https://github.com/takemo101/sine-mml/releases) ページを開く
2. 「Draft a new release」をクリック
3. タグ `v0.2.0` を選択
4. リリースタイトル: `v0.2.0`
5. リリースノートを記入（変更内容、新機能、バグ修正など）
6. バイナリをアップロード（下記参照）
7. 「Publish release」をクリック

---

## 🔨 バイナリのビルド

### macOS（Apple Silicon）

```bash
cargo build --release
# 出力: target/release/sine-mml
```

### macOS（Intel）

```bash
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
# 出力: target/x86_64-apple-darwin/release/sine-mml
```

### Linux（x86_64）

```bash
# Docker を使用（クロスコンパイル）
docker run --rm -v "$(pwd)":/app -w /app rust:latest \
  cargo build --release

# または cross を使用
cargo install cross
cross build --release --target x86_64-unknown-linux-gnu
```

### Windows（x86_64）

```bash
# クロスコンパイル
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
# 出力: target/x86_64-pc-windows-gnu/release/sine-mml.exe
```

### バイナリ命名規則

| プラットフォーム | ファイル名 |
|-----------------|-----------|
| macOS (ARM) | `sine-mml-v0.2.0-aarch64-apple-darwin` |
| macOS (Intel) | `sine-mml-v0.2.0-x86_64-apple-darwin` |
| Linux | `sine-mml-v0.2.0-x86_64-unknown-linux-gnu` |
| Windows | `sine-mml-v0.2.0-x86_64-pc-windows-gnu.exe` |

---

## 📦 crates.io への公開（オプション）

### 事前準備

1. [crates.io](https://crates.io/) アカウントを作成
2. APIトークンを取得: `cargo login`

### 公開

```bash
# ドライラン（実際には公開しない）
cargo publish --dry-run

# 公開
cargo publish
```

### 注意事項

- `Cargo.toml` の `repository`、`license`、`description` が正しく設定されていること
- `README.md` が存在すること
- 一度公開したバージョンは削除できません

---

## 🔄 リリース後

1. **Issueのクローズ**: リリースに含まれるIssueをクローズ
2. **アナウンス**: 必要に応じてSNS等でアナウンス
3. **次期バージョンの準備**: 次のマイルストーンを設定

---

## 📋 リリースノートのテンプレート

```markdown
## v0.2.0 (2026-01-10)

### ✨ 新機能
- ループ再生機能を追加 (#15)
- メトロノーム機能を追加 (#18)

### 🐛 バグ修正
- 音量設定が反映されない問題を修正 (#20)

### 📚 ドキュメント
- README.md を日本語化
- USAGE.md を追加

### 🔧 内部改善
- 依存関係を更新
```

---

質問があれば Issue でお気軽にどうぞ！🎵
