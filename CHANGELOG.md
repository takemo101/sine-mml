# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.3] - 2026-01-17

### Fixed

- `--version`オプションがCargo.tomlのバージョンを自動取得するよう修正 (#201)
  - 従来はハードコードされた"0.1.0"が表示されていた問題を修正

## [0.2.2] - 2026-01-17

### Fixed

- `--midi-out`オプション使用時に不要なMML表示を削除 (#199)
  - 通常の`play`コマンドと挙動を統一

## [0.2.1] - 2026-01-16

### Fixed

- メトロノームがMML内のテンポ変更に追従するよう修正 (#197)
  - `T120 CDEF T180 GHAB` のようにテンポを途中で変更した場合、メトロノームのクリック間隔も追従

### Added

- MMLサンプルファイル追加
  - `examples/beethoven-style.mml` - ベートーヴェン交響曲第5番風
  - `examples/beethoven-tempest.mml` - 激しいテンペスト風
  - `examples/chopin-2026.mml` - 現代のショパン風ノクターン

## [0.2.0] - 2026-01-16

### Added

- **MIDIストリーミング** (`--midi-out`, `--midi-channel`, `--midi-list`)
  - 外部MIDIデバイス/DAWへリアルタイム送信
  - MIDIチャンネル選択（1-16）
  - デバイス一覧表示（`midi list`サブコマンド対応）
  - ドリフトフリーの "Next Event Time" タイミング
  - Ctrl-C割り込み時のAll Notes Off送信
- **MIDIループ再生** (`--loop-play`オプション併用)
  - MIDI出力でのループ再生対応
- **MIDIプログレスバー**
  - MIDI出力時の進捗表示
- **`--no-history / -N`オプション**
  - 履歴保存をスキップ
- **連符（n連符）** (`{...}n` 構文)
  - 複数音符を指定拍に均等配分
  - ベース音長指定対応（`{...}n:m`）
  - ネスト対応（最大5階層）
- **`midi list`サブコマンド**
  - MMLなしでMIDIデバイス一覧表示

### Changed

- テスト分離によるコード品質向上
  - parser.rs, synthesizer.rs, ast.rs, player.rs, message.rs, mod.rs
  - 各ファイルを500行以下に削減

### Fixed

- flakyテストの安定化（MIDI progress, db tests）

## [0.1.1] - 2026-01-12

### Added

- **タイ記号機能** (`&`)
  - 音符を連結して長い音を表現（例: `C4&8` = 1.5拍）
  - 複数連結対応（例: `C4&8&16`）
  - 休符のタイも対応（例: `R4&8`）
  - 付点音符との組み合わせ対応
- **MMLファイル読み込み機能** (`--file`オプション)
  - `.mml`ファイルからMML文字列を読み込み
  - `#`で始まる行をコメントとして除去
  - 空行を無視
  - UTF-8エンコーディング必須
  - ファイルサイズ上限1MB（DoS攻撃防止）
- **相対ボリューム指定** (`V+n`, `V-n`)
  - 現在のボリュームからの相対的な増減が可能
  - `V+`/`V-`でデフォルト増減値（±1）
  - 範囲外の値は0-15にクランプ
- **ループネスト対応** (最大5階層)
  - `[ CDE [ FGAB ]2 ]3`のような入れ子ループに対応
  - ネスト内でも脱出ポイント（`:`）使用可能
  - 総展開コマンド数上限10,000（DoS攻撃防止）
- **ループ構文** (`[...]n`)
  - 繰り返しフレーズを簡潔に記述
  - 脱出ポイント（`:`）で1番カッコ・2番カッコ的な表現
- **履歴メモ機能** (`--note`オプション)
  - 履歴に最大500文字のメモを付与可能
- **履歴削除機能** (`clear-history`コマンド)
  - 全履歴を削除（確認プロンプト付き）
- **小文字MML記述対応**
  - 小文字でMMLコマンドを記述可能（自動正規化）
- **メトロノーム機能**: ノイズベースのクリック音生成
  - `--metronome` フラグでメトロノームを有効化
  - `--metronome-beat` で4/8/16ビートを選択
  - `--metronome-volume` で音量調節（0.0〜1.0）
- **音声ノーマライゼーション**: クリッピング防止機能
- **E2E統合テスト基盤**: `assert_cmd`を使用したCLIテスト
- **CI/CDパイプライン**: GitHub Actionsによる自動テスト・ビルド

### Changed

- **デフォルトボリューム**: V10に変更
- **履歴保存タイミング**: ループ再生時も再生前に履歴を保存するよう変更

### Removed

- **`--bpm`オプション**: MML内の`T`コマンドに統合（Breaking Change）
  - 移行方法: `--bpm 180` → MML内に`T180`を記述

### Fixed

- メトロノームオプションが機能しない問題を修正
- オクターブチェンジ（`>` `<`）のパースエラーを修正
- Clippy警告の修正
- E2Eテストのタイムアウト問題を修正

## [2.1.0] - 2026-01-12

### Added

- **MMLファイル読み込み機能** (`--file`オプション)
  - `.mml`ファイルからMML文字列を読み込み
  - `#`で始まる行をコメントとして除去
  - 空行を無視
  - UTF-8エンコーディング必須
  - ファイルサイズ上限1MB（DoS攻撃防止）
- **相対ボリューム指定** (`V+n`, `V-n`)
  - 現在のボリュームからの相対的な増減が可能
  - `V+`/`V-`でデフォルト増減値（±1）
  - 範囲外の値は0-15にクランプ
- **ループネスト対応** (最大5階層)
  - `[ CDE [ FGAB ]2 ]3`のような入れ子ループに対応
  - ネスト内でも脱出ポイント（`:`）使用可能
  - 総展開コマンド数上限10,000（DoS攻撃防止）
- **ループ構文** (`[...]n`)
  - 繰り返しフレーズを簡潔に記述
  - 脱出ポイント（`:`）で1番カッコ・2番カッコ的な表現
- **履歴メモ機能** (`--note`オプション)
  - 履歴に最大500文字のメモを付与可能
- **履歴削除機能** (`clear-history`コマンド)
  - 全履歴を削除（確認プロンプト付き）
- **小文字MML記述対応**
  - 小文字でMMLコマンドを記述可能（自動正規化）

### Changed

- **デフォルトボリューム**: V10に変更

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

[0.2.0]: https://github.com/takemo101/sine-mml/releases/tag/v0.2.0
[0.1.1]: https://github.com/takemo101/sine-mml/releases/tag/v0.1.1
[0.1.0]: https://github.com/takemo101/sine-mml/releases/tag/v0.1.0
