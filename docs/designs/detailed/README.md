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

---

## REQ-CLI-002（機能改善）詳細設計書一覧

### 関連ドキュメント
- **要件定義書**: [REQ-CLI-002_MML-Synthesizer-Enhancement.md](../../requirements/REQ-CLI-002_MML-Synthesizer-Enhancement.md)
- **基本設計書**: [BASIC-CLI-002_MML-Synthesizer-Enhancement.md](../basic/BASIC-CLI-002_MML-Synthesizer-Enhancement.md)

### 詳細設計書一覧（v2.0）

| ドキュメントID | 名称 | ディレクトリ | 関連機能ID | ステータス | GitHub Issue |
|----------------|------|--------------|-----------|------------|--------------|
| DET-MET-001 | メトロノーム機能 | [metronome/](./metronome/) | F-015/016/017 | 作成済み | #29, #30, #31 |
| DET-NRM-001 | 音声ノーマライゼーション | [normalizer/](./normalizer/) | F-019 | 作成済み | #28 |
| DET-LOOP-001 | ループ履歴タイミング変更 | [loop-history/](./loop-history/) | F-018 | 作成済み | #32 |
| DET-CLI-002 | CLIオプション変更（BPM削除） | [cli-options/](./cli-options/) | F-005/016/017 | 作成済み | #35 |
| DET-E2E-001 | E2Eテスト基盤 | [e2e-test/](./e2e-test/) | F-021/022 | 作成済み | #36 |
| DET-CAP-001 | CLI-Backend対応マトリクス | [capabilities/](./capabilities/) | F-020 | 作成済み | #33 |

### テスト項目書

| ドキュメント | 説明 | テストケース数 |
|-------------|------|---------------|
| [テスト項目書](./test-spec/テスト項目書.md) | v2.0全機能のテストケース | 42件 |

### Epic Issue

- [#27 [Epic] MML Synthesizer CLI 機能改善 v2.0 (BASIC-CLI-002)](https://github.com/takemo101/sine-mml/issues/27)

### 実装順序（推奨）

依存関係を考慮し、以下の順序で実装することを推奨する。

1. **DET-NRM-001** (#28) - ノーマライザー（基盤として最優先）
2. **DET-MET-001** (#29) - ノイズベースクリック音生成
3. **DET-MET-002** (#30) - ビート間隔計算
4. **DET-MET-003** (#31) - メトロノームミックス実装
5. **DET-LOOP-001** (#32) - ループ履歴タイミング変更
6. **DET-CLI-002** (#35) - CLIオプション変更（破壊的変更）
7. **DET-E2E-001** (#36) - E2Eテスト基盤構築
8. **DET-CAP-001** (#33) - CLI-Backend対応マトリクス
9. **ドキュメント更新** (#34)

---

## 変更履歴

| 日付 | バージョン | 変更内容 | 作成者 |
|------|------------|----------|--------|
| 2026-01-10 | 1.0.0 | 初版作成、フォルダ構造定義 | Antigravity |
| 2026-01-11 | 1.1.0 | REQ-CLI-002（機能改善）詳細設計書フォルダ追加 | basic-design-writer |
| 2026-01-11 | 2.0.0 | v2.0詳細設計書作成完了、GitHub Issue登録 | detailed-design-writer |
