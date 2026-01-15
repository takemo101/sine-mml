# MML サンプルファイル

このディレクトリには sine-mml で再生できるサンプル MML ファイルが含まれています。

## ファイル一覧

| ファイル | 説明 | おすすめ波形 |
|---------|------|-------------|
| `twinkle-star.mml` | きらきら星 | sine |
| `scale.mml` | Cメジャースケール | sine |
| `arpeggio.mml` | アルペジオパターン（相対ボリューム使用） | sawtooth |
| `chime.mml` | チャイム音 | sine |
| `retro-game.mml` | レトロゲーム風 | square |
| `dynamics.mml` | ダイナミクス（音量変化）の例 | sine |
| `nested-loop.mml` | ネストループの例 | sine |
| `tuplet.mml` | 連符（n連符）の基本例（v3.0新機能） | sine |
| `tuplet-advanced.mml` | 連符の応用例（ネスト、休符、ループ組み合わせ） | sine |

## 使い方

```bash
# 基本的な再生
sine-mml play --file examples/twinkle-star.mml

# 波形を指定して再生
sine-mml play --file examples/retro-game.mml --waveform square

# メモ付きで履歴に保存
sine-mml play --file examples/arpeggio.mml --note "アルペジオ練習"

# メトロノーム付きで再生
sine-mml play --file examples/scale.mml --metronome --metronome-beat 8
```

## MML 構文クイックリファレンス

- `CDEFGAB` - 音符
- `R` - 休符
- `#` / `+` - シャープ
- `-` / `b` - フラット
- `O1-O8` - オクターブ設定
- `>` / `<` - オクターブ上げ/下げ
- `L1-L64` - デフォルト音長
- `T30-T300` - テンポ
- `V0-V15` - 音量（絶対値）
- `V+n` / `V-n` - 音量（相対値）
- `[...]n` - ループ（n回繰り返し）
- `[...:...]n` - 脱出ポイント付きループ
- `{...}n` - 連符（n連符）
- `{...}n:m` - ベース音長指定の連符
- `.` - 付点
- `&` - タイ記号（音符を連結）

詳細は [USAGE.md](../USAGE.md) を参照してください。
