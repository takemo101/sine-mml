# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - 2026-01-12

### Added

- **メトロノーム機能**: ノイズベースのクリック音生成
  - `--metronome` フラグでメトロノームを有効化
  - `--metronome-beat` で4/8/16ビートを選択
  - `--metronome-volume` で音量調節（0.0〜1.0）
- **音声ノーマライゼーション**: クリッピング防止機能
- **E2E統合テスト基盤**: `assert_cmd`を使用したCLIテスト
- **CLI-Backend対応マトリクス**: 機能対応状況のドキュメント化

### Changed

- **履歴保存タイミング**: ループ再生時も再生前に履歴を保存するよう変更

### Removed

- **`--bpm`オプション**: MML内の`T`コマンドに統合（Breaking Change）
  - 移行方法: `--bpm 180` → MML内に`T180`を記述

## [0.1.0] - 2026-01-10

### Added

- **MML Parser**: Complete MML (Music Macro Language) parser implementation
  - Note parsing (C, D, E, F, G, A, B) with accidentals (#, +, -, b)
  - Rest notation (R) with duration support
  - Octave control (O, <, >)
  - Tempo setting (T)
  - Default length (L)
  - Volume control (V)
  - Dot notation for extended durations

- **Audio Engine**: High-performance audio synthesis
  - Waveform generation (sine, sawtooth, square)
  - ADSR envelope support
  - Real-time playback via cpal
  - WAV file export

- **Database**: SQLite-based history management
  - Schema initialization
  - CRUD operations for play history

- **CLI**: Full command-line interface
  - `play` command: Play MML strings with waveform selection
  - `history` command: View play history
  - `export` command: Export history to WAV files
  - Progress bar display during playback

- **Documentation**: Japanese documentation
  - README.md with usage examples
  - USAGE.md with detailed MML syntax reference
  - CONTRIBUTING.md for contributors
  - RELEASE.md for release process

- **Build**: Development tooling
  - Justfile with common tasks (build, test, lint, demo)
  - CI workflow configuration

[0.1.0]: https://github.com/takemo101/sine-mml/releases/tag/v0.1.0
