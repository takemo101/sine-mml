# 🤝 コントリビューションガイド

sine-mml への貢献を歓迎します！このドキュメントでは、プロジェクトへの参加方法を説明します。

---

## 🚀 開発を始める

### 1. リポジトリをフォーク・クローン

```bash
git clone https://github.com/YOUR_USERNAME/sine-mml.git
cd sine-mml
```

### 2. 依存関係のインストール

**Linux の場合**:
```bash
sudo apt-get install libasound2-dev
```

**macOS / Windows**: 追加の依存関係は不要です。

### 3. ビルド・テスト

```bash
# ビルド
cargo build

# テスト実行
cargo test

# Lint チェック（pedantic）
cargo clippy -- -D warnings

# フォーマット
cargo fmt
```

---

## 🔀 開発ワークフロー

### ブランチ命名規則

| プレフィックス | 用途 | 例 |
|---------------|------|-----|
| `feature/` | 新機能 | `feature/issue-15-loop-playback` |
| `fix/` | バグ修正 | `fix/issue-20-volume-range` |
| `refactor/` | リファクタリング | `refactor/issue-25-parser` |
| `docs/` | ドキュメント | `docs/add-usage-examples` |

### コミットメッセージ規約

```
<type>: <description>

[optional body]

[optional footer]
```

**type の種類**:
- `feat`: 新機能
- `fix`: バグ修正
- `refactor`: リファクタリング
- `test`: テスト追加・修正
- `docs`: ドキュメント

**例**:
```
feat: ループ再生機能を追加

--loop-play オプションでループ再生が可能に。
Ctrl+C で停止。

Closes #15
```

### プルリクエスト

1. フォークから新しいブランチを作成
2. 変更を実装
3. テストが通ることを確認
4. プルリクエストを作成
5. レビューを受けて修正

---

## 📋 コード規約

### 必須チェック

| チェック | コマンド | 要件 |
|---------|---------|------|
| ビルド | `cargo build` | 成功すること |
| テスト | `cargo test` | 全テスト通過 |
| Lint | `cargo clippy -- -D warnings` | 警告なし |
| フォーマット | `cargo fmt --check` | 差分なし |

### スタイルガイド

- **ドキュメントコメント**: 公開関数には `///` でドキュメントを記述
- **エラーハンドリング**: `thiserror` を使用したカスタムエラー型
- **unsafe禁止**: `#![forbid(unsafe_code)]` が設定されています

---

## 🏗 プロジェクト構成

```
src/
├── main.rs          # エントリーポイント
├── lib.rs           # ライブラリルート
├── mml/             # MMLパーサー・AST
│   ├── ast.rs       # 抽象構文木
│   ├── parser.rs    # パーサー
│   └── error.rs     # パースエラー
├── audio/           # オーディオエンジン
│   ├── synthesizer.rs  # 音声合成
│   ├── waveform.rs     # 波形生成
│   ├── player.rs       # 再生
│   └── exporter.rs     # WAVエクスポート
├── db/              # データベース
│   ├── schema.rs    # スキーマ定義
│   └── history.rs   # 履歴管理
└── cli/             # CLIインターフェース
    ├── args.rs      # 引数定義
    ├── handlers.rs  # コマンドハンドラ
    └── output.rs    # 出力フォーマット
```

---

## 🐛 Issue報告

### バグ報告

以下の情報を含めてください：

- **現象**: 何が起きたか
- **期待動作**: 何が起きるべきか
- **再現手順**: どうすれば再現できるか
- **環境**: OS、Rustバージョン

### 機能リクエスト

- **概要**: どんな機能が欲しいか
- **ユースケース**: なぜその機能が必要か
- **代替案**: 他の解決策はあるか

---

## 📄 ライセンス

貢献していただいたコードは、プロジェクトと同じ **MITライセンス** の下で公開されます。

---

ご質問があれば、Issueでお気軽にお問い合わせください！🎵
