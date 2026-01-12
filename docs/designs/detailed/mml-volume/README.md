# Relative Volume Specification - 詳細設計

## 概要

F-028: 相対ボリューム指定機能の詳細設計書を格納するディレクトリ

## 関連ドキュメント

- 基本設計書: [BASIC-CLI-004](../../basic/BASIC-CLI-004_MML-Advanced-Features.md)
- 要件定義書: [REQ-CLI-004](../../../requirements/REQ-CLI-004_MML-Advanced-Features.md)

## 詳細設計書一覧

| ファイル名 | 概要 | ステータス |
|-----------|------|-----------|
| relative-volume.md | 相対ボリューム解析・計算の詳細設計 | 未着手 |

## 機能概要

- ボリューム範囲: V0-V15（既存仕様維持）
- 絶対値指定: `V10`
- 相対値指定: `V+2`, `V-3`
- デフォルト増減: `V+`, `V-` (±1)
- 範囲外はクランプ（0-15）
- デフォルト値: V10
