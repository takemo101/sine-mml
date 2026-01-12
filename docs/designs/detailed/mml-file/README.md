# MML File Reading - 詳細設計

## 概要

F-027: MMLファイル読み取り機能の詳細設計書を格納するディレクトリ

## 関連ドキュメント

- 基本設計書: [BASIC-CLI-004](../../basic/BASIC-CLI-004_MML-Advanced-Features.md)
- 要件定義書: [REQ-CLI-004](../../../requirements/REQ-CLI-004_MML-Advanced-Features.md)

## 詳細設計書一覧

| ファイル名 | 概要 | ステータス |
|-----------|------|-----------|
| file-reader.md | ファイル読み取り処理の詳細設計 | 未着手 |

## 機能概要

- `--file` オプションでMMLファイルを指定
- `.mml` 拡張子のみ対応
- `#` で始まる行はコメントとして除去
- 空行は無視
- ファイルサイズ上限: 1MB
- UTF-8 エンコーディング必須
