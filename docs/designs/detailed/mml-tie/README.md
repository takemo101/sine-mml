# MML タイ記号機能 詳細設計書

## 概要

MML（Music Macro Language）のタイ記号 `&` 機能の詳細設計書群。

## 関連ドキュメント

| ドキュメント | 説明 |
|-------------|------|
| [要件定義書](../../../requirements/REQ-CLI-005_Tie-Notation.md) | 機能要件、ビジネスルール、テストケース |
| [基本設計書](../../basic/BASIC-CLI-005_Tie-Notation.md) | システムアーキテクチャ、モジュール設計 |

## 詳細設計書一覧

| # | ファイル | 対象 | 概要 |
|---|---------|------|------|
| 1 | [token-definition.md](./token-definition.md) | `src/mml/mod.rs` | Token::Tie の追加、トークナイザー修正 |
| 2 | [ast-extension.md](./ast-extension.md) | `src/mml/ast.rs` | TiedDuration構造体、Note/Rest拡張 |
| 3 | [parser-implementation.md](./parser-implementation.md) | `src/mml/parser.rs` | parse_note/parse_rest のタイ処理 |
| 4 | [duration-calculation.md](./duration-calculation.md) | `src/mml/ast.rs` | タイ音長計算ロジック |
| 5 | [error-handling.md](./error-handling.md) | `src/mml/error.rs` | 3種のタイエラー型 |

## 実装順序

```
1. token-definition.md    → Token::Tie 追加
2. error-handling.md      → エラー型追加（パーサーより先に必要）
3. ast-extension.md       → TiedDuration, Note/Rest 拡張
4. duration-calculation.md → 音長計算メソッド
5. parser-implementation.md → パーサー実装（最後、全ての依存を解決）
```

## 対応する要件

### 機能要件

| ID | 要件 | 対応設計書 |
|----|------|-----------|
| F-030 | MMLタイ記号 | 全て |

### ビジネスルール

| ID | ルール | 対応設計書 |
|----|--------|-----------|
| BR-084 | 同一音程のみ連結可 | parser-implementation.md |
| BR-085 | 異なる音程はエラー | error-handling.md |
| BR-086 | 休符のタイ許可 | parser-implementation.md |
| BR-087 | タイ後に付点可 | parser-implementation.md |
| BR-088 | 連鎖無制限 | parser-implementation.md |
| BR-089 | 空白許容 | token-definition.md |

### テストケース

| ID | ケース | 対応設計書 |
|----|--------|-----------|
| TC-030-001 | C4&8 | duration-calculation.md |
| TC-030-002 | C8&8 | duration-calculation.md |
| TC-030-003 | C4&4 | duration-calculation.md |
| TC-030-004 | C4.&8 | duration-calculation.md |
| TC-030-005 | C4&8. | duration-calculation.md |
| TC-030-006 | C4&8&16 | duration-calculation.md |
| TC-030-007 | R4&8 | duration-calculation.md |
| TC-030-008 | C1&1 | duration-calculation.md |
| TC-030-009 | C4 & 8 | token-definition.md |
| TC-030-010 | C4&D4 エラー | error-handling.md |
| TC-030-011 | C4& エラー | error-handling.md |
| TC-030-012 | C4&R4 エラー | error-handling.md |
| TC-030-013 | &C4 エラー | parser-implementation.md |

## 変更ファイル一覧

| ファイル | 変更種別 | 変更内容 |
|---------|---------|---------|
| `src/mml/mod.rs` | 修正 | Token::Tie 追加、トークナイザー修正 |
| `src/mml/ast.rs` | 修正 | TiedDuration追加、Note/Rest拡張、音長計算 |
| `src/mml/parser.rs` | 修正 | parse_note/parse_rest タイ処理追加 |
| `src/mml/error.rs` | 修正 | 3種のエラー型追加 |

## ステータス

| 設計書 | ステータス |
|--------|----------|
| token-definition.md | ✅ 完了 |
| ast-extension.md | ✅ 完了 |
| parser-implementation.md | ✅ 完了 |
| duration-calculation.md | ✅ 完了 |
| error-handling.md | ✅ 完了 |

---

**作成日**: 2026-01-12  
**バージョン**: 1.0.0
