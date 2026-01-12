# 🎵 sine-mml

[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Test](https://github.com/takemo101/sine-mml/actions/workflows/test.yml/badge.svg)](https://github.com/takemo101/sine-mml/actions/workflows/test.yml)
[![Lint](https://github.com/takemo101/sine-mml/actions/workflows/lint.yml/badge.svg)](https://github.com/takemo101/sine-mml/actions/workflows/lint.yml)
[![Build](https://github.com/takemo101/sine-mml/actions/workflows/build.yml/badge.svg)](https://github.com/takemo101/sine-mml/actions/workflows/build.yml)

**sine-mml** は、**MML（Music Macro Language）** を使って音楽を合成・再生するためのコマンドラインツールです。Rustで構築された高性能なオーディオエンジンにより、MML文字列から様々な波形で音楽を再生できます。

---

## 🚀 概要

レトロゲーム音楽が好きな方、ゲーム開発者、音声合成に興味がある方に最適なツールです：

- **再生**: MML文字列をターミナルから直接再生
- **ファイル読み込み**: MMLファイル（`.mml`）からの読み込み（v2.1新機能）
- **波形選択**: サイン波、ノコギリ波、矩形波から選択
- **ループ構文**: 繰り返しフレーズを簡潔に記述（ネスト対応、v2.1新機能）
- **相対ボリューム**: `V+n`, `V-n`で音量を動的に増減（v2.1新機能）
- **メトロノーム**: ノイズベースのクリック音で練習をサポート（v2.0新機能）
- **履歴管理**: SQLiteによる永続的な演奏履歴の管理（メモ付き）
- **エクスポート**: 演奏をWAVファイルとして出力

---

## 🛠 インストール

### 必要条件

- **Rustツールチェーン**: [Rust](https://www.rust-lang.org/tools/install) がインストールされていること（edition 2021）
- **オーディオ依存関係（Linuxのみ）**:
  ```bash
  sudo apt-get install libasound2-dev
  ```

### ソースからビルド

```bash
git clone https://github.com/takemo101/sine-mml.git
cd sine-mml
cargo install --path .
```

---

## 🎹 クイックスタート

```bash
# Cメジャースケールを再生
sine-mml play "CDEFGAB >C"

# ノコギリ波でテンポ180で再生（テンポはMML内のTコマンドで指定）
sine-mml play "T180 L8 O5 C D E F G A B >C" --waveform sawtooth

# ループ構文でフレーズを繰り返し（v2.1新機能）
sine-mml play "T120 [CDEF]3 G2"  # CDEFを3回繰り返してからG

# 脱出ポイント付きループ（v2.1新機能）
sine-mml play "[CD:EF]2"  # 1回目: CDEF、2回目: CD（EFをスキップ）

# ネストしたループ（v2.1新機能、最大5階層）
sine-mml play "[ CDE [ FGAB ]2 ]3"  # 内側を2回、外側を3回

# MMLファイルから再生（v2.1新機能）
sine-mml play --file song.mml

# 相対ボリュームで音量を変化（v2.1新機能）
sine-mml play "V10 C V+2 D V-3 E"  # V10 → V12 → V9

# 履歴にメモを付けて再生（v2.1新機能）
sine-mml play "CDEFGAB" --note "練習用スケール"

# メトロノーム付きで再生（v2.0新機能）
sine-mml play "T120 CDEFGAB" --metronome --metronome-beat 8 --metronome-volume 0.5

# 履歴を表示
sine-mml history

# 履歴から再生（ID指定）
sine-mml play --history-id 1

# 履歴をWAVファイルとしてエクスポート
sine-mml export --history-id 1 --output my_music.wav

# 全履歴を削除（v2.1新機能）
sine-mml clear-history
```

詳細な使い方は [USAGE.md](USAGE.md) を参照してください。

> **v2.0移行ノート**: `--bpm`オプションは削除されました。テンポはMML内の`T`コマンドで指定してください（例: `T140 CDEFGAB`）。

---

## 📖 コマンド一覧

| コマンド | 説明 |
|---------|------|
| `play` | MML文字列を合成・再生 |
| `history` | 演奏履歴を表示 |
| `export` | 履歴をWAVファイルとして出力 |
| `clear-history` | 全履歴を削除（v2.1新機能） |

---

## 🎼 MML構文リファレンス

| コマンド | 説明 | 例 |
|---------|------|-----|
| `CDEFGAB` | 基本音符 | `C D E` |
| `#` / `+` | シャープ | `C#` / `C+` |
| `-` / `b` | フラット | `Eb` / `E-` |
| `R` | 休符 | `R4`（4分休符） |
| `On` | オクターブ設定（1-8） | `O5` |
| `>` / `<` | オクターブ上げ/下げ | `>C` |
| `Ln` | デフォルト音長（1-64） | `L8`（8分音符） |
| `Tn` | テンポ（30-300 BPM） | `T140` |
| `Vn` | 音量（0-15） | `V10` |
| `V+n` / `V-n` | 相対ボリューム（v2.1新機能） | `V+2`（+2）、`V-3`（-3） |
| `.` | 付点音符 | `C4.` |
| `&` | タイ記号（v2.2新機能） | `C4&8`（1.5拍の音符） |
| `[...]n` | ループ（1-99回、ネスト5階層まで） | `[CDEF]3`（3回繰り返し） |
| `[...:...]n` | 脱出ポイント付きループ | `[CD:EF]2`（2回目はEFをスキップ） |

**例**: `"T120 L4 O4 CDE R G >C"` - 120BPMで4分音符のC, D, E、休符、G、次のオクターブのCを再生

### タイ記号（v2.2新機能）

タイ記号（`&`）を使用して、複数の音価を連結し、より長い音符を表現できます。

**構文:**
```
<音符><音価>&<音価>[&<音価>...]
```

**例:**
- `C4&8` - 4分音符と8分音符を連結（1.5拍）
- `C4&8&16` - 3つの音符を連結（1.75拍）
- `R4&8` - 休符のタイ（1.5拍の休符）
- `C4.&8` - 付点音符とのタイ（2.0拍）

**制約:**
- タイで連結できるのは音価（数値）のみです
- `C4&D4` のような異なる音符を連結することはできません
- 休符のタイも同様に音価のみを連結します

---

## 🛠 開発

```bash
# ビルド
cargo build

# テスト
cargo test

# Lint
cargo clippy -- -D warnings
```

開発に参加したい方は [CONTRIBUTING.md](CONTRIBUTING.md) を参照してください。

---

## 📄 ライセンス

このプロジェクトは **MITライセンス** の下で公開されています。詳細は [LICENSE](LICENSE) ファイルを参照してください。

---

作成者: **[takemo101](https://github.com/takemo101)** 🎶
