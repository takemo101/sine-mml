# MML構文拡張 詳細設計

## 概要

BASIC-CLI-003に基づくMML構文拡張の詳細設計ドキュメント群。

## 対象機能

| 機能ID | 機能名 | 詳細設計書 | バックエンド設計書 |
|--------|--------|-----------|------------------|
| F-023 | MMLループ構文 | [詳細設計書.md](./詳細設計書.md) | [バックエンド設計書.md](./バックエンド設計書.md) |
| F-024 | 小文字MML記述 | - (実装済み) | - |

## 関連ドキュメント

- 基本設計書: [BASIC-CLI-003_MML-Syntax-Extension.md](../../basic/BASIC-CLI-003_MML-Syntax-Extension.md)
- 要件定義書: [REQ-CLI-003_MML-Syntax-Extension.md](../../../requirements/REQ-CLI-003_MML-Syntax-Extension.md)

## 影響を受けるモジュール

- `src/mml/mod.rs` - ループ構文トークン追加 (`[`, `]`, `:`)
- `src/mml/parser.rs` - ループ構文解析
- `src/mml/ast.rs` - `Command::Loop`バリアント追加
- `src/mml/error.rs` - ループ関連エラー追加

## 重要な発見

**F-024（小文字MML記述）は既に実装済み**

`src/mml/mod.rs:56` の `tokenize()` 関数で `c.to_ascii_uppercase()` が実装されており、小文字入力は既に大文字に正規化されています。テスト `tokenize_case_insensitive` も存在します。

```rust
let token = match c.to_ascii_uppercase() {
    'C' | 'D' | 'E' | 'F' | 'G' | 'A' | 'B' => {
        // ...
    }
    // ...
}
```

## ステータス

| ドキュメント | ステータス |
|------------|-----------|
| 詳細設計書.md (F-023) | ✅ 作成済み |
| バックエンド設計書.md (F-023) | ✅ 作成済み |
| F-024 関連ドキュメント | - (既存実装で対応済み) |

## 変更履歴

| 日付 | バージョン | 変更内容 |
|------|------------|----------|
| 2026-01-11 | 1.0.0 | 初版作成（詳細設計書・バックエンド設計書） |
