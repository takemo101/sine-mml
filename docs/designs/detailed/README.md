# 詳細設計書インデックス

## 概要
本ディレクトリには、MML Synthesizer CLI（sine-mml）の詳細設計書を格納する。

## 関連ドキュメント
- **要件定義書**: [REQ-CLI-001_MML-Synthesizer.md](../../requirements/REQ-CLI-001_MML-Synthesizer.md)
- **基本設計書**: [BASIC-CLI-001_MML-Synthesizer.md](../basic/BASIC-CLI-001_MML-Synthesizer.md)
- **技術調査レポート**: [TECH-REPORT-20260110_Combined.md](../../research/TECH-REPORT-20260110_Combined.md)

## 詳細設計書一覧

| ドキュメントID | 名称 | ディレクトリ | ステータス |
|----------------|------|--------------|------------|
| DET-MML-001 | MMLパーサー詳細設計 | [mml-parser/](./mml-parser/) | 作成済み |
| DET-AUD-001 | オーディオエンジン詳細設計 | [audio-engine/](./audio-engine/) | 作成済み |
| DET-DB-001 | データベース詳細設計 | [database/](./database/) | 作成済み |
| DET-CLI-001 | CLIインターフェース詳細設計 | [cli-interface/](./cli-interface/) | 作成済み |
| DET-SEC-001 | セキュリティ設計 | [common/](./common/) | 作成済み |
| DET-INFRA-001 | インフラ設計 | [common/](./common/) | 作成済み |

## ディレクトリ構成

- `docs/designs/detailed/`
  - `README.md` - このファイル（インデックス）
  - `mml-parser/` - MMLパーサー詳細設計
    - `詳細設計書.md`
    - `バックエンド設計書.md`
  - `audio-engine/` - オーディオエンジン詳細設計
    - `詳細設計書.md`
    - `バックエンド設計書.md`
  - `database/` - データベース詳細設計
    - `詳細設計書.md`
    - `バックエンド設計書.md`
    - `データベース設計書.md`
  - `cli-interface/` - CLIインターフェース詳細設計
    - `詳細設計書.md`
    - `バックエンド設計書.md`
    - `画面設計書.md`
  - `common/` - 共通設計書
    - `セキュリティ設計書.md`
    - `インフラ設計書.md`

## 作成順序（推奨）

依存関係を考慮し、以下の順序で詳細設計書を作成することを推奨する。

1. **DET-MML-001** (MMLパーサー) - 他モジュールへの依存なし、入力の起点
2. **DET-DB-001** (データベース) - 他モジュールへの依存なし
3. **DET-AUD-001** (オーディオエンジン) - MMLパーサーのAST出力に依存
4. **DET-CLI-001** (CLIインターフェース) - 全モジュールを統合

## 変更履歴

| 日付 | バージョン | 変更内容 | 作成者 |
|------|------------|----------|--------|
| 2026-01-10 | 1.0.0 | 初版作成、フォルダ構造定義 | Antigravity |
