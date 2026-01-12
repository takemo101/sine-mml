# Loop Nesting Support - 詳細設計

## 概要

F-029: ループネスト対応機能の詳細設計書を格納するディレクトリ

## 関連ドキュメント

- 基本設計書: [BASIC-CLI-004](../../basic/BASIC-CLI-004_MML-Advanced-Features.md)
- 要件定義書: [REQ-CLI-004](../../../requirements/REQ-CLI-004_MML-Advanced-Features.md)
- 既存ループ設計: [BASIC-CLI-003](../../basic/BASIC-CLI-003_MML-Syntax-Extension.md)

## 詳細設計書一覧

| ファイル名 | 概要 | ステータス |
|-----------|------|-----------|
| loop-nesting.md | ループネスト解析・展開の詳細設計 | 未着手 |

## 機能概要

- 最大ネスト深度: 5階層
- 6階層以上はエラー
- ネスト内でも脱出ポイント（`:`）使用可能
- ループ回数: 各階層で1-99
- 総展開コマンド数上限: 10,000

## BASIC-CLI-003 からの変更点

- `Parser` に `loop_depth` フィールド追加
- `parse_loop()` でネスト深度チェック追加
- `expand_loop()` を再帰的展開に変更
- 新規エラー型: `LoopNestTooDeep`, `LoopExpandedTooLarge`
