# 📖 使い方ガイド

このドキュメントでは、sine-mml の詳細な使い方を説明します。

---

## 目次

1. [基本的な使い方](#基本的な使い方)
2. [playコマンド](#playコマンド)
3. [historyコマンド](#historyコマンド)
4. [exportコマンド](#exportコマンド)
5. [clear-historyコマンド](#clear-historyコマンド)
6. [MML構文詳細](#mml構文詳細)
7. [MMLファイル読み込み](#mmlファイル読み込み)
8. [相対ボリューム指定](#相対ボリューム指定)
9. [ループ構文](#ループ構文)
10. [サンプル曲](#サンプル曲)
11. [トラブルシューティング](#トラブルシューティング)

---

## 基本的な使い方

### ヘルプを表示

```bash
sine-mml --help
sine-mml play --help
```

### バージョンを確認

```bash
sine-mml --version
```

---

## playコマンド

MML文字列を解析し、音声を合成・再生します。

### 基本構文

```bash
sine-mml play <MML文字列> [オプション]
```

### オプション一覧

| オプション | 短縮形 | 説明 | デフォルト |
|-----------|-------|------|-----------|
| `--waveform` | `-w` | 波形タイプ（sine/sawtooth/square） | sine |
| `--volume` | `-v` | 音量（0.0〜1.0） | 1.0 |
| `--loop-play` | - | ループ再生（Ctrl+Cで停止） | false |
| `--metronome` | - | メトロノーム音を追加 | false |
| `--metronome-beat` | - | メトロノームのビート（4/8/16） | 4 |
| `--metronome-volume` | - | メトロノームの音量（0.0〜1.0） | 0.5 |
| `--history-id` | - | 履歴IDから再生 | - |
| `--note` | - | 履歴に付けるメモ（最大500文字）（v2.1新機能） | - |

> **Note**: v2.0で`--bpm`オプションは削除されました。テンポはMML内の`T`コマンドで指定してください（例: `T140`）。

### 使用例

```bash
# シンプルな音階を再生
sine-mml play "CDEFGAB"

# ノコギリ波で再生
sine-mml play "CDEFGAB" --waveform sawtooth

# 音量を半分にして再生
sine-mml play "CDEFGAB" --volume 0.5

# テンポ180で再生（MML内のTコマンドで指定）
sine-mml play "T180 CDEFGAB"

# ループ構文を使用（v2.1新機能）
sine-mml play "[CDEF]3 G2"  # CDEFを3回繰り返してからG

# 履歴にメモを付けて再生（v2.1新機能）
sine-mml play "CDEFGAB" --note "練習用スケール"

# ループ再生（Ctrl+Cで停止）
sine-mml play "CDEFGAB" --loop-play

# 矩形波、音量0.3、テンポ200で再生
sine-mml play "T200 CDEFGAB" -w square -v 0.3

# メトロノーム付きで再生（8ビート、音量0.3）
sine-mml play "T120 CDEFGAB" --metronome --metronome-beat 8 --metronome-volume 0.3

# 履歴ID 5 を再生
sine-mml play --history-id 5
```

### 波形の違い

| 波形 | 説明 | 音色 |
|-----|------|------|
| `sine` | サイン波 | 柔らかく純粋な音 |
| `sawtooth` | ノコギリ波 | 明るく鋭い音 |
| `square` | 矩形波 | レトロゲーム風の音 |

---

## historyコマンド

過去の演奏履歴を一覧表示します。

### 基本構文

```bash
sine-mml history
```

### 出力例

```
┌────┬──────────────────────────┬──────────┬────────┬─────────────────────┬────────────────┐
│ ID │ MML                      │ Waveform │ Volume │ Created At          │ Note           │
├────┼──────────────────────────┼──────────┼────────┼─────────────────────┼────────────────┤
│ 5  │ T140 L8 O5 CDEFGAB       │ sine     │ 1.00   │ 2026-01-10 20:30:15 │ 練習用スケール │
│ 4  │ CDEFGAB                  │ sawtooth │ 0.50   │ 2026-01-10 20:15:30 │                │
│ 3  │ O4 L4 C D E F G A B >C   │ square   │ 0.80   │ 2026-01-10 19:45:00 │ テスト         │
└────┴──────────────────────────┴──────────┴────────┴─────────────────────┴────────────────┘
```

> **Note**: v2.1から `Note` 列が追加されました。`--note` オプションで付けたメモが表示されます。

### 履歴の再生

```bash
# 履歴ID 5 を再生
sine-mml play --history-id 5
```

---

## exportコマンド

履歴をWAVファイルとしてエクスポートします。

### 基本構文

```bash
sine-mml export --history-id <ID> --output <ファイルパス>
```

### オプション

| オプション | 短縮形 | 説明 | 必須 |
|-----------|-------|------|-----|
| `--history-id` | - | エクスポートする履歴ID | ✅ |
| `--output` | `-o` | 出力ファイルパス | ✅ |

### 使用例

```bash
# 履歴ID 5 を my_music.wav として保存
sine-mml export --history-id 5 --output my_music.wav

# 絶対パスで指定
sine-mml export --history-id 5 -o /Users/username/Music/output.wav
```

### 出力形式

- **フォーマット**: WAV（PCM）
- **サンプルレート**: 44,100 Hz
- **ビット深度**: 16bit
- **チャンネル**: モノラル

---

## clear-historyコマンド

全ての演奏履歴を削除します。（v2.1新機能）

### 基本構文

```bash
sine-mml clear-history
```

### 動作

1. コマンドを実行すると、確認プロンプトが表示されます
2. `y` または `yes` を入力すると、全ての履歴が削除されます
3. その他の入力では、削除がキャンセルされます

### 使用例

```bash
$ sine-mml clear-history
全ての履歴を削除しますか？ [y/N]: y
履歴を削除しました。

$ sine-mml clear-history
全ての履歴を削除しますか？ [y/N]: n
キャンセルしました。
```

> **警告**: この操作は取り消せません。削除された履歴は復元できません。

---

## MML構文詳細

### 音符

| 記号 | 説明 | 例 |
|-----|------|-----|
| `C D E F G A B` | 音符（ドレミファソラシ） | `CDE` |
| `c d e f g a b` | 小文字も使用可能（v2.1） | `cde` |
| `R` | 休符 | `R4` |

### 音長（デフォルト: 4分音符）

| 記号 | 説明 | 例 |
|-----|------|-----|
| `1` | 全音符 | `C1` |
| `2` | 2分音符 | `C2` |
| `4` | 4分音符 | `C4` |
| `8` | 8分音符 | `C8` |
| `16` | 16分音符 | `C16` |
| `32` | 32分音符 | `C32` |
| `64` | 64分音符 | `C64` |
| `.` | 付点（1.5倍） | `C4.` |

### 変化記号

| 記号 | 説明 | 例 |
|-----|------|-----|
| `#` / `+` | シャープ（半音上げ） | `C#` / `C+` |
| `-` / `b` | フラット（半音下げ） | `Eb` / `E-` |

### オクターブ

| 記号 | 説明 | 例 |
|-----|------|-----|
| `O1`〜`O8` | オクターブ設定 | `O5 C` |
| `>` | オクターブを1つ上げる | `C >C` |
| `<` | オクターブを1つ下げる | `C <C` |

### テンポ・音量・音長設定

| 記号 | 説明 | 範囲 | 例 |
|-----|------|------|-----|
| `Tn` | テンポ設定 | 30〜300 | `T140` |
| `Vn` | 音量設定 | 0〜15 | `V10` |
| `Ln` | デフォルト音長設定 | 1〜64 | `L8` |

---

## ループ構文

v2.1で追加されたループ構文を使用すると、繰り返しフレーズを簡潔に記述できます。

### 基本構文

```
[繰り返す内容]回数
```

角括弧 `[]` で囲んだ内容を、指定した回数だけ繰り返します。

### 使用例

```bash
# CDEFを3回繰り返す
sine-mml play "[CDEF]3"
# 展開後: CDEF CDEF CDEF

# ループとその後のフレーズを組み合わせる
sine-mml play "[CDEF]2 G2"
# 展開後: CDEF CDEF G2
```

### 脱出ポイント

ループ内にコロン `:` を入れると、最終回のみコロン以降をスキップします。
これは音楽の「1番カッコ・2番カッコ」のような表現に便利です。

```
[前半:後半]回数
```

| 回数 | 実行される部分 |
|-----|---------------|
| 1〜(n-1)回目 | 前半 + 後半 |
| n回目（最終回） | 前半のみ |

### 脱出ポイントの使用例

```bash
# 2回ループ、2回目はEFをスキップ
sine-mml play "[CD:EF]2"
# 1回目: CDEF
# 2回目: CD
# 展開後: CDEF CD

# 3回ループ、3回目だけ異なる
sine-mml play "[CDEF:G]3 A2"
# 1回目: CDEFG
# 2回目: CDEFG
# 3回目: CDEF
# 展開後: CDEFG CDEFG CDEF A2
```

### 制限事項

| 制限 | 説明 |
|-----|------|
| ループ回数 | 1〜99回（範囲外はエラー） |
| ネスト | ループのネストは不可（`[[...]]`はエラー） |
| 脱出ポイント | ループ内に1つのみ（複数はエラー） |

### エラー例

```bash
# エラー: ループ回数が範囲外
sine-mml play "[CDEF]0"   # 0回は無効
sine-mml play "[CDEF]100" # 100回は無効

# エラー: ネストループ
sine-mml play "[[CD]2 EF]2"

# エラー: 閉じ括弧がない
sine-mml play "[CDEF"

# エラー: ループ外での脱出ポイント
sine-mml play "CD:EF"

# エラー: 複数の脱出ポイント
sine-mml play "[CD:EF:G]2"
```

---

## サンプル曲

### きらきら星

```bash
sine-mml play "T120 L4 O4 CCGGAAG2 FFEEDDC2 GGFFEED2 GGFFEED2 CCGGAAG2 FFEEDDC2"
```

### きらきら星（ループ構文版）

```bash
sine-mml play "T120 L4 O4 CCGGAAG2 FFEEDDC2 [GGFFEED2]2 CCGGAAG2 FFEEDDC2"
```

### ドレミの歌

```bash
sine-mml play "T120 L4 O4 C8D8 E C E C E2 D8E8F8F8 E D F2"
```

### チャイム音

```bash
sine-mml play "T100 L4 O5 E C D G2 R4 G D E C2" -w sine -v 0.6
```

### レトロゲーム風

```bash
sine-mml play "T180 L8 O5 CDEFG4 R8 GFEDC4" -w square -v 0.5
```

### アルペジオ

```bash
sine-mml play "T140 L16 O4 CEG>C<GEC CEG>C<GEC" -w sawtooth
```

### アルペジオ（ループ構文版）

```bash
sine-mml play "T140 L16 O4 [CEG>C<GEC]2" -w sawtooth
```

---

## トラブルシューティング

### 音が出ない

**原因**: オーディオデバイスが見つからない

**対処法**:
```bash
# Linux: ALSAがインストールされているか確認
aplay -l

# macOS: サウンド設定を確認
# システム環境設定 → サウンド → 出力デバイス
```

### 「履歴が見つかりません」エラー

**原因**: 指定した履歴IDが存在しない

**対処法**:
```bash
# 履歴一覧を確認
sine-mml history

# 存在するIDを指定
sine-mml play --history-id <存在するID>
```

### WAVエクスポートが失敗する

**原因**: 出力パスが無効、または書き込み権限がない

**対処法**:
```bash
# 絶対パスで指定
sine-mml export --history-id 1 -o /tmp/output.wav

# 書き込み権限を確認
ls -la /path/to/directory
```

### MMLパースエラー

**原因**: MML構文が不正

**対処法**:
- 音符は大文字または小文字（C, D, E... または c, d, e...）で記述
- 数値は適切な範囲内か確認（テンポ: 30-300、音量: 0-15、ループ: 1-99）
- シャープは `#` または `+`、フラットは `-` または `b`
- ループは `[]` で囲み、閉じ括弧の後に回数を指定

### ループ構文のエラー

**原因**: ループ構文が不正

**対処法**:
- 開き括弧 `[` と閉じ括弧 `]` が対応しているか確認
- ループ回数が1〜99の範囲内か確認
- ネストループ（`[[...]]`）を使用していないか確認
- 脱出ポイント `:` がループ内に1つだけか確認

---

## データの保存場所

履歴データはSQLiteデータベースに保存されます：

| OS | パス |
|----|------|
| macOS | `~/Library/Application Support/sine-mml/sine-mml.db` |
| Linux | `~/.local/share/sine-mml/sine-mml.db` |
| Windows | `%APPDATA%\sine-mml\sine-mml.db` |

### データベースのリセット

```bash
# macOS/Linux
rm -rf ~/.local/share/sine-mml/

# または
rm -rf ~/Library/Application\ Support/sine-mml/
```

> **Note**: v2.1以降は `clear-history` コマンドで履歴のみを削除できます。

---

## v2.0 移行ガイド

### `--bpm`オプションの廃止

v2.0で`--bpm`オプションは削除されました。テンポはMML内の`T`コマンドで指定してください。

**Before (v1.x)**:
```bash
sine-mml play "CDEFGAB" --bpm 180
```

**After (v2.0)**:
```bash
sine-mml play "T180 CDEFGAB"
```

この変更により、MML文字列が自己完結的になり、履歴からの再生時も同じテンポで再生されます。

---

ご質問があれば、[GitHub Issues](https://github.com/takemo101/sine-mml/issues) でお気軽にどうぞ！🎵
