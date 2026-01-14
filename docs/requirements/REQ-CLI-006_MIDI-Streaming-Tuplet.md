# MML Synthesizer CLI MIDIストリーミング・連符機能 要件定義書

## メタ情報

| 項目 | 内容 |
|------|------|
| ドキュメントID | REQ-CLI-006 |
| バージョン | 1.0.1 |
| ステータス | ドラフト |
| 作成日 | 2026-01-14 |
| 最終更新日 | 2026-01-14 |
| 作成者 | req-writer |
| 承認者 | - |
| 関連文書 | REQ-CLI-001_MML-Synthesizer.md (v1.1.0)<br>REQ-CLI-005_Tie-Notation.md (v1.0.1)<br>docs/memos/MIDIストリーミング機能追加.md |

---

## 1. プロジェクト概要

### 1.1 背景

sine-mml v2.2のタイ記号機能実装完了後、さらなる音楽表現力の向上と外部機器連携のため、以下の機能追加が必要となりました：

1. **MIDIストリーミング機能の欠如**: 現状、sine-mmlは内蔵シンセサイザーでのみ再生可能で、外部MIDI機器やソフトウェアシンセサイザーとの連携ができない。REQ-CLI-001では「MIDI入出力: 対象外」としていたが、ユーザーからの要望により対象範囲に変更
2. **連符（n連符）表現の欠如**: 3連符、5連符などの連符を表現する手段がなく、複雑なリズムパターンを記述できない
3. **楽譜との互換性**: 一般的な楽譜では連符が頻繁に使用されるが、MMLでは同等の機能がない

これらの改善により、MMLの音楽表現力を大幅に向上させ、外部機器との連携を可能にします。

### 1.2 目的

- MIDIストリーミング機能により、外部MIDI機器やソフトウェアシンセサイザーでMMLを演奏可能にする
- 連符構文（`{...}n`）により、3連符、5連符などの複雑なリズムを表現可能にする
- MMLのテンポ設定に従った正確なMIDIメッセージ送信を実現する
- 既存のループ構文との一貫性を保ちつつ、直感的な連符記法を提供する

### 1.3 ゴール

| 目標 | 成功指標 |
|------|---------|
| MIDIストリーミング実装 | `--midi-out` オプションでMIDIデバイスにリアルタイム送信される |
| MIDIデバイス選択 | 複数のMIDI出力先から選択可能 |
| 連符構文の実装 | `{CDE}3` で3連符が正確に演奏される |
| ループとの一貫性 | `[...]n` と `{...}n` の構文が統一されている |
| エラー検出 | 不正な連符指定が適切なエラーメッセージで検出される |

### 1.4 スコープ

#### 対象範囲
- MIDIストリーミング機能（MIDI出力のみ）
  - `--midi-out` オプションの追加
  - MIDIデバイス選択機能
  - リアルタイムMIDIメッセージ送信
  - MIDIエラーハンドリング
- 連符（n連符）機能
  - `{...}n` ブラケット構文の追加
  - 3連符、4連符、5連符などの任意のn連符対応
  - 休符を含む連符
  - タイ記号との組み合わせ
  - ネスト対応（最大5階層）

#### 対象外
- MIDI入力（MIDIキーボードからの入力）
- MIDIファイル（.mid）の入出力
- MIDI CC（コントロールチェンジ）メッセージ
- MIDIクロック同期
- 連符のネスト深度6階層以上

---

## 2. ステークホルダー

### 2.1 ステークホルダー一覧

| 役割 | 担当者/部門 | 関心事 | 影響度 |
|------|------------|--------|--------|
| プロダクトオーナー | - | 外部機器連携、音楽表現力の向上 | 高 |
| 開発チーム | - | MIDI実装の複雑性、パーサー拡張 | 高 |
| エンドユーザー | 音楽制作者、趣味プログラマー | 外部シンセサイザーの利用、複雑なリズム表現 | 高 |

### 2.2 ユーザーペルソナ

#### ペルソナ1: 外部シンセサイザーを使いたい作曲者
| 項目 | 内容 |
|------|------|
| 属性 | 30-50代、音楽制作経験豊富 |
| 課題 | sine-mmlの内蔵シンセサイザーでは音色が限定的で、外部のハードウェア/ソフトウェアシンセサイザーを使いたい |
| ニーズ | MMLで記述したメロディを外部MIDI機器で演奏したい |
| 利用シーン | DAWと連携してMMLをMIDI入力として使用、ハードウェアシンセサイザーでの演奏 |

#### ペルソナ2: 複雑なリズムを表現したい作曲者
| 項目 | 内容 |
|------|------|
| 属性 | 20-40代、音楽理論に詳しい |
| 課題 | 3連符や5連符などの連符を表現する手段がなく、ジャズやクラシック音楽の楽譜を再現できない |
| ニーズ | 連符構文で複雑なリズムパターンを記述したい |
| 利用シーン | ジャズのスウィング、クラシック音楽の楽譜入力、ポリリズムの作成 |

#### ペルソナ3: 楽譜を忠実に再現したい作曲者
| 項目 | 内容 |
|------|------|
| 属性 | 30代、音楽経験あり |
| 課題 | 楽譜の連符をMMLで表現できない |
| ニーズ | 楽譜通りの連符を記述し、正確に演奏したい |
| 利用シーン | クラシック音楽やジャズの楽譜をMMLで入力 |

---

## 3. 機能要件

### 3.1 機能一覧

**※ REQ-CLI-001（F-001〜F-014）、REQ-CLI-002（F-015〜F-022）、REQ-CLI-003（F-023〜F-026）、REQ-CLI-004（F-027〜F-029）、REQ-CLI-005（F-030）との連番を維持**

| ID | 機能名 | 概要 | 優先度 | フェーズ | 備考 |
|----|--------|------|--------|---------|------|
| F-031 | MIDIストリーミング | `--midi-out` でMIDIデバイスにリアルタイム送信 | 必須 | Phase 2.3 | **新規** |
| F-032 | 連符（n連符） | `{...}n` で連符を表現 | 必須 | Phase 2.3 | **新規** |

### 3.2 ユーザーストーリー

#### US-019: MIDIデバイスで演奏したい
- **ユーザー**: 作曲者
- **したいこと**: MMLで記述したメロディを外部MIDI機器やソフトウェアシンセサイザーで演奏したい
- **理由**: 内蔵シンセサイザーでは音色が限定的で、外部の高品質なシンセサイザーを使いたい
- **受け入れ基準**:
  - [ ] `sine-mml play "CDEFGAB" --midi-out` でMIDIデバイスにメッセージが送信される
  - [ ] MIDIデバイスが複数ある場合、選択可能
  - [ ] `--midi-channel 10` でMIDIチャンネル10（ドラム等）を指定可能
  - [ ] MMLのテンポ設定（`T120`）に従って正確にMIDIメッセージが送信される
  - [ ] 音量（`V10`）がMIDIベロシティに変換される
  - [ ] MIDIデバイスが接続されていない場合、適切なエラーメッセージが表示される
  - [ ] `--midi-out` と `--waveform` を同時に指定した場合、内蔵シンセサイザーとMIDI出力の両方で演奏される
  - [ ] Ctrl+Cで演奏を中断すると、MIDIノートオフメッセージが送信される
  - [ ] 無効なMIDIチャンネル（0, 17等）を指定した場合、エラーメッセージが表示される
- **関連機能**: F-031

#### US-020: 連符を表現したい
- **ユーザー**: 作曲者
- **したいこと**: 3連符、5連符などの連符をMMLで表現したい
- **理由**: ジャズやクラシック音楽の楽譜を再現したい
- **受け入れ基準**:
  - [ ] `{CDE}3` で3連符（3音を1拍に収める）が演奏される
  - [ ] `{CDEF}4` で4連符が演奏される
  - [ ] `L8 {CDE}3` で8分音符ベースの3連符が演奏される
  - [ ] `{CDE}3:2` で2分音符に3音を収める連符が演奏される
  - [ ] `{CDR}3` で休符を含む連符が演奏される
  - [ ] `{C4&8 D E}3` でタイ記号を含む連符が演奏される
  - [ ] `[{CDE}3]2` でループと連符の組み合わせが演奏される
  - [ ] `{{CDE}3 FG}5` でネストした連符が演奏される（最大5階層）
  - [ ] `{CDE}` のように連符数が指定されていない場合、エラーメッセージが表示される
  - [ ] `{CDE}0` や `{CDE}1` のように不正な連符数が指定された場合、エラーメッセージが表示される
- **関連機能**: F-032

### 3.3 機能詳細

#### F-031: MIDIストリーミング

**概要**: `--midi-out` オプションでMIDIデバイスにリアルタイムでMIDIメッセージを送信

**入力**:
- MML文字列
- `--midi-out` オプション（デバイス名またはデバイスID）
- `--midi-channel` オプション（MIDIチャンネル: 1-16、デフォルト: 1）
- `--midi-list` オプション（利用可能なMIDIデバイス一覧を表示）

**出力**:
- MIDIノートオン/オフメッセージ
- MIDIベロシティ（音量）
- MIDIチャンネル（指定されたチャンネル、デフォルト: 1）

**処理概要**:
1. 利用可能なMIDIデバイスを列挙
2. `--midi-out` で指定されたデバイスに接続
3. MMLを解析し、音符データに変換
4. 音符ごとにMIDIノートオンメッセージを送信
5. 音長に応じた待機
6. MIDIノートオフメッセージを送信
7. 演奏終了またはCtrl+Cで全ノートオフメッセージを送信

**ビジネスルール**:
- BR-090: MIDIチャンネルは `--midi-channel` オプションで指定可能（1-16、デフォルト: 1）
- BR-091: MML音量（V0-V15）をMIDIベロシティ（0-127）に変換（V0=0, V15=127, 線形補間）
- BR-092: MIDIノートナンバーはMMLオクターブと音程から計算（C4=60）
- BR-093: MIDIデバイスが接続されていない場合はエラー
- BR-094: 演奏中断時は全ノートオフメッセージを送信してクリーンアップ
- BR-104: 無効なMIDIチャンネル（0以下または17以上）はエラー

**制約事項**:
- MIDI入力は非対応
- MIDIファイル（.mid）の入出力は非対応
- MIDI CC（コントロールチェンジ）メッセージは非対応
- MIDIクロック同期は非対応

**技術実装のポイント**:

**1. MIDI クレート選定**:
```toml
# Cargo.toml
[dependencies]
midir = "0.9"  # クロスプラットフォームMIDI I/O
```

**2. MIDIデバイス列挙**:
```rust
use midir::{MidiOutput, MidiOutputConnection};

pub fn list_midi_devices() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let midi_out = MidiOutput::new("sine-mml")?;
    let ports = midi_out.ports();
    
    let mut devices = Vec::new();
    for (i, port) in ports.iter().enumerate() {
        let name = midi_out.port_name(port)?;
        devices.push(format!("{}: {}", i, name));
    }
    
    Ok(devices)
}
```

**3. MIDIデバイス接続**:
```rust
pub fn connect_midi_device(device_id: usize) -> Result<MidiOutputConnection, Box<dyn std::error::Error>> {
    let midi_out = MidiOutput::new("sine-mml")?;
    let ports = midi_out.ports();
    
    if device_id >= ports.len() {
        return Err("Invalid MIDI device ID".into());
    }
    
    let port = &ports[device_id];
    let conn = midi_out.connect(port, "sine-mml-output")?;
    
    Ok(conn)
}
```

**4. MIDIメッセージ送信**:
```rust
pub fn send_note_on(
    conn: &mut MidiOutputConnection,
    channel: u8,
    note: u8,
    velocity: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let msg = [0x90 | (channel - 1), note, velocity];
    conn.send(&msg)?;
    Ok(())
}

pub fn send_note_off(
    conn: &mut MidiOutputConnection,
    channel: u8,
    note: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let msg = [0x80 | (channel - 1), note, 0];
    conn.send(&msg)?;
    Ok(())
}
```

**5. MML音程からMIDIノートナンバー変換**:
```rust
pub fn mml_to_midi_note(pitch: Pitch, accidental: Option<Accidental>, octave: u8) -> u8 {
    let base = match pitch {
        Pitch::C => 0,
        Pitch::D => 2,
        Pitch::E => 4,
        Pitch::F => 5,
        Pitch::G => 7,
        Pitch::A => 9,
        Pitch::B => 11,
    };
    
    let offset = match accidental {
        Some(Accidental::Sharp) => 1,
        Some(Accidental::Flat) => -1,
        None => 0,
    };
    
    // C4 = MIDI note 60
    ((octave as i16 + 1) * 12 + base + offset) as u8
}
```

**6. MML音量からMIDIベロシティ変換**:
```rust
pub fn mml_volume_to_velocity(volume: u8) -> u8 {
    // V0-V15 -> 0-127
    ((volume as f32 / 15.0) * 127.0) as u8
}
```

**7. エラー型追加**:
```rust
// src/midi/error.rs
#[derive(Debug, thiserror::Error)]
pub enum MidiError {
    /// MML-E015: MIDIデバイスが見つからない
    #[error("[MML-E015] MIDIデバイスが見つかりません")]
    NoDeviceFound,
    
    /// MML-E016: MIDIデバイス接続エラー
    #[error("[MML-E016] MIDIデバイスへの接続に失敗しました: {0}")]
    ConnectionFailed(String),
    
    /// MML-E017: MIDIメッセージ送信エラー
    #[error("[MML-E017] MIDIメッセージの送信に失敗しました: {0}")]
    SendFailed(String),
    
    /// MML-E018: 無効なMIDIデバイスID
    #[error("[MML-E018] 無効なMIDIデバイスIDです: {0}")]
    InvalidDeviceId(usize),
    
    /// MML-E019: MIDIデバイス切断
    #[error("[MML-E019] MIDIデバイスが切断されました")]
    DeviceDisconnected,
    
    /// MML-E024: 無効なMIDIチャンネル
    #[error("[MML-E024] 無効なMIDIチャンネルです（1-16を指定してください）: {0}")]
    InvalidChannel(u8),
}
```

**受け入れテスト例**:
```rust
#[test]
fn test_midi_note_conversion() {
    // C4 = MIDI note 60
    assert_eq!(mml_to_midi_note(Pitch::C, None, 4), 60);
    // A4 = MIDI note 69
    assert_eq!(mml_to_midi_note(Pitch::A, None, 4), 69);
    // C#5 = MIDI note 73
    assert_eq!(mml_to_midi_note(Pitch::C, Some(Accidental::Sharp), 5), 73);
}

#[test]
fn test_velocity_conversion() {
    assert_eq!(mml_volume_to_velocity(0), 0);
    assert_eq!(mml_volume_to_velocity(15), 127);
    assert_eq!(mml_volume_to_velocity(8), 67);  // 中間値
}
```

---

#### F-032: 連符（n連符）

**概要**: `{...}n` ブラケット構文で連符を表現

**入力**:
- MML文字列（連符を含む）
  - 例: `{CDE}3`, `{CDEF}4`, `L8 {CDE}3`, `{CDE}3:2`

**出力**:
- 連符で指定された音符の音長を調整した音

**処理概要**:
1. トークナイザーで `{` と `}` を認識
2. パーサーで `{...}n` の構文を解析
3. 括弧内の音符を取得
4. 連符数（n）を取得
5. ベース音長を取得（デフォルト音長または `:` 後の指定）
6. 各音符の音長を `ベース音長 / n` に調整
7. 調整された音長で音を発音

**ビジネスルール**:
- BR-095: 連符数（n）は2以上の整数
- BR-096: 連符のベース音長はデフォルト音長（`L4` など）または `:` 後の指定
- BR-097: 連符内の音符は個別に音長を指定可能（`{C4 D8 E}3` など）
- BR-098: 連符内に休符を含めることが可能（`{CDR}3`）
- BR-099: 連符内にタイ記号を含めることが可能（`{C4&8 D E}3`）
- BR-100: 連符のネスト深度は最大5階層（ループと同様）
- BR-101: 連符とループの組み合わせが可能（`[{CDE}3]2`）
- BR-102: 連符数が指定されていない場合はエラー（`{CDE}` は不可）
- BR-103: 連符数が0または1の場合はエラー

**制約事項**:
- 連符のネスト深度6階層以上は不可
- 連符数は99まで（実用上の制限）

**技術実装のポイント**:

**1. トークン定義**:
```rust
// src/mml/mod.rs
pub enum Token {
    // 既存のトークン...
    TupletStart,  // {
    TupletEnd,    // }
    Colon,        // :
}
```

**2. AST拡張**:
```rust
// src/mml/ast.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    // 既存のコマンド...
    Tuplet {
        notes: Vec<Command>,
        count: u8,
        base_duration: Option<u8>,  // : 後の指定
    },
}
```

**3. パーサー実装**:
```rust
// src/mml/parser.rs
fn parse_tuplet(&mut self) -> Result<Command, ParseError> {
    self.consume(Token::TupletStart)?;  // { を消費
    
    let mut notes = Vec::new();
    
    // 括弧内の音符を解析
    while !self.check(Token::TupletEnd) {
        if self.is_eof() {
            return Err(ParseError::UnclosedTuplet {
                position: self.pos,
            });
        }
        
        let cmd = self.parse_command()?;
        notes.push(cmd);
    }
    
    self.consume(Token::TupletEnd)?;  // } を消費
    
    // 連符数を取得
    if !self.current_char().is_ascii_digit() {
        return Err(ParseError::TupletCountMissing {
            position: self.pos,
        });
    }
    
    let count = self.parse_number()?;
    
    if count < 2 {
        return Err(ParseError::InvalidTupletCount {
            count,
            position: self.pos,
        });
    }
    
    // ベース音長の指定（オプション）
    let base_duration = if self.check(Token::Colon) {
        self.advance();  // : を消費
        Some(self.parse_number()?)
    } else {
        None
    };
    
    Ok(Command::Tuplet {
        notes,
        count,
        base_duration,
    })
}
```

**4. シンセサイザー実装**:
```rust
// src/audio/synthesizer.rs
impl Command {
    pub fn duration_in_seconds(&self, bpm: u16, default_length: u8) -> f32 {
        match self {
            Command::Tuplet { notes, count, base_duration } => {
                // ベース音長を取得
                let base = base_duration.unwrap_or(default_length);
                let base_seconds = 60.0 / bpm as f32 * 4.0 / base as f32;
                
                // 各音符の音長を調整
                let tuplet_duration = base_seconds / *count as f32;
                
                // 全音符の合計時間
                notes.iter()
                    .map(|note| {
                        // 音符が個別に音長を指定している場合は考慮
                        match note {
                            Command::Note(n) => {
                                if n.duration.is_some() {
                                    n.duration_in_seconds(bpm, default_length) / *count as f32
                                } else {
                                    tuplet_duration
                                }
                            },
                            Command::Rest(r) => {
                                if r.duration.is_some() {
                                    r.duration_in_seconds(bpm, default_length) / *count as f32
                                } else {
                                    tuplet_duration
                                }
                            },
                            _ => 0.0,
                        }
                    })
                    .sum()
            },
            // 既存の処理...
        }
    }
}
```

**5. エラー型追加**:
```rust
// src/mml/error.rs
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    // 既存のエラー...
    
    /// MML-E020: 連符の閉じ括弧がない
    #[error("[MML-E020] 連符の閉じ括弧 '}}' がありません: 位置 {position}")]
    UnclosedTuplet {
        position: usize,
    },
    
    /// MML-E021: 連符数が指定されていない
    #[error("[MML-E021] 連符数が指定されていません: 位置 {position}")]
    TupletCountMissing {
        position: usize,
    },
    
    /// MML-E022: 無効な連符数
    #[error("[MML-E022] 無効な連符数です（2以上を指定してください）: {count}, 位置 {position}")]
    InvalidTupletCount {
        count: u8,
        position: usize,
    },
    
    /// MML-E023: 連符のネスト深度超過
    #[error("[MML-E023] 連符のネスト深度が最大値（5階層）を超えています: 位置 {position}")]
    TupletNestTooDeep {
        position: usize,
    },
}
```

**受け入れテスト例**:
```rust
#[test]
fn test_tuplet_basic() {
    let mml = parse("{CDE}3").unwrap();
    // 3連符: 3音を1拍（4分音符）に収める
    // 各音符の長さ = 1/4拍 / 3 = 1/12拍
}

#[test]
fn test_tuplet_with_default_length() {
    let mml = parse("L8 {CDE}3").unwrap();
    // 8分音符ベースの3連符
    // 各音符の長さ = 1/8拍 / 3 = 1/24拍
}

#[test]
fn test_tuplet_with_base_duration() {
    let mml = parse("{CDE}3:2").unwrap();
    // 2分音符に3音を収める
    // 各音符の長さ = 2拍 / 3 = 2/3拍
}

#[test]
fn test_tuplet_with_rest() {
    let mml = parse("{CDR}3").unwrap();
    // 休符を含む3連符
}

#[test]
fn test_tuplet_with_tie() {
    let mml = parse("{C4&8 D E}3").unwrap();
    // タイ記号を含む3連符
}

#[test]
fn test_tuplet_with_loop() {
    let mml = parse("[{CDE}3]2").unwrap();
    // ループと連符の組み合わせ
}

#[test]
fn test_tuplet_nested() {
    let mml = parse("{{CDE}3 FG}5").unwrap();
    // ネストした連符
}

#[test]
fn test_tuplet_no_count() {
    let result = parse("{CDE}");
    assert!(result.is_err());
    // Error: 連符数が指定されていません
}

#[test]
fn test_tuplet_invalid_count() {
    let result = parse("{CDE}1");
    assert!(result.is_err());
    // Error: 無効な連符数です（2以上を指定してください）
}
```

**エラーメッセージ例**:
```
Error: 連符の閉じ括弧 '}' がありません: 位置 5
  {CDE
      ^
Expected: '}'

Error: 連符数が指定されていません: 位置 5
  {CDE}
      ^
Expected: 数字（2以上）

Error: 無効な連符数です（2以上を指定してください）: 1, 位置 6
  {CDE}1
       ^
Expected: 2以上の数字
```

---

## 4. 非機能要件

### 4.1 性能要件

| ID | 要件 | 目標値 | 測定方法 |
|----|------|--------|----------|
| NFR-P-016 | MIDIメッセージ送信レイテンシ | 5ms以内 | MIDIタイムスタンプ測定 |
| NFR-P-017 | 連符解析のオーバーヘッド | 10ms以内（100個の連符） | ベンチマーク |
| NFR-P-018 | MIDI接続確立時間 | 100ms以内 | 接続時間測定 |

### 4.2 可用性要件

| ID | 要件 | 目標値 |
|----|------|--------|
| NFR-A-010 | MIDIエラーのハンドリング | 詳細なエラーメッセージを表示 |
| NFR-A-011 | MIDI切断時の復旧 | 自動再接続（オプション） |

### 4.3 セキュリティ要件

| ID | 要件 | 詳細 |
|----|------|------|
| NFR-S-013 | 連符のネスト深度制限 | 最大5階層 |
| NFR-S-014 | 連符数の制限 | 2〜99 |

### 4.4 ユーザビリティ要件

| ID | 要件 | 詳細 |
|----|------|------|
| NFR-U-016 | MIDIデバイス一覧表示 | `--midi-list` で利用可能なデバイスを表示 |
| NFR-U-017 | 連符のエラーメッセージ | 位置情報と修正ヒントを明示 |
| NFR-U-018 | MIDI接続状態表示 | 接続成功/失敗を明確に表示 |

### 4.5 保守性要件

| ID | 要件 | 詳細 |
|----|------|------|
| NFR-M-016 | MIDIのテスト | 正常系、異常系を網羅 |
| NFR-M-017 | 連符のテスト | 正常系、異常系、ネストを網羅 |
| NFR-M-018 | 後方互換性 | 既存のMMLが正常に動作 |

---

## 5. 制約条件

### 5.1 技術的制約

| 制約 | 詳細 | 理由 |
|------|------|------|
| MIDIライブラリ | midir | クロスプラットフォーム対応、活発なメンテナンス |
| MIDI出力のみ | MIDI入力は非対応 | Phase 2.3のスコープ外 |
| 連符のネスト深度 | 最大5階層 | ループと同様の制限 |

### 5.2 ビジネス制約

| 制約 | 詳細 |
|------|------|
| 予算 | オープンソース（無償） |
| スケジュール | Phase 2.3: 3週間 |
| リソース | 個人開発（1名） |

### 5.3 法規制・コンプライアンス

| 要件 | 詳細 |
|------|------|
| ライセンス | MIT License（変更なし） |
| 後方互換性 | 既存のMMLが正常に動作することを保証 |

---

## 6. 外部インターフェース

### 6.1 既存機能への影響

| 機能ID | 機能名 | 影響内容 | 対応 |
|--------|--------|----------|------|
| F-001 | MMLパーサー | **拡張** | 連符構文を追加 |
| F-002 | リアルタイム再生 | **拡張** | MIDI出力を追加 |

### 6.2 MML構文の拡張

#### 追加される構文
```
{<音符>...}n         # 連符
{<音符>...}n:m       # ベース音長指定の連符
```

#### 例
```
# 基本的な連符
{CDE}3           # 3連符: 3音を1拍に収める

# デフォルト音長との連動
L8 {CDE}3        # 8分音符ベースの3連符

# ベース音長指定
{CDE}3:2         # 2分音符に3音を収める

# 休符を含む連符
{CDR}3           # 3連符の3音目が休符

# タイとの組み合わせ
{C4&8 D E}3      # 連符内でタイ記号を使用

# ループとの組み合わせ
[{CDE}3]2        # 3連符を2回繰り返し

# ネスト
{{CDE}3 FG}5     # ネストした連符
```

### 6.3 CLIオプションの追加

#### 新規オプション
```bash
# MIDIデバイス一覧表示
sine-mml --midi-list

# MIDIストリーミング（デバイスID指定）
sine-mml play "CDEFGAB" --midi-out 0

# MIDIストリーミング（デバイス名指定）
sine-mml play "CDEFGAB" --midi-out "IAC Driver Bus 1"

# MIDIチャンネル指定（1-16、デフォルト: 1）
sine-mml play "CDEFGAB" --midi-out 0 --midi-channel 2

# ドラムチャンネル（ch10）を指定
sine-mml play "C4 D4 E4 F4" --midi-out 0 --midi-channel 10

# 内蔵シンセサイザーとMIDI出力の両方
sine-mml play "CDEFGAB" --waveform sine --midi-out 0
```

---

## 7. 前提条件と依存関係

### 7.1 前提条件

- REQ-CLI-001の全機能が実装済み（v1.0.0リリース済み）
- REQ-CLI-002の全機能が実装済み（v2.0.0リリース済み）
- REQ-CLI-003の全機能が実装済み（v2.1.0リリース済み）
- REQ-CLI-004の全機能が実装済み（v2.1.0リリース済み）
- REQ-CLI-005の全機能が実装済み（v2.2.0リリース済み）

### 7.2 依存関係

| 依存先 | 内容 | 影響 |
|--------|------|------|
| MMLパーサー | 連符構文の解析 | パーサー拡張 |
| シンセサイザー | 連符音長の計算 | 音長計算ロジック拡張 |
| MIDIライブラリ | MIDI出力 | 新規依存 |

### 7.3 新規依存クレート

| クレート | バージョン | 用途 |
|---------|----------|------|
| midir | 0.9+ | クロスプラットフォームMIDI I/O |

---

## 8. リスクと課題

### 8.1 リスク一覧

| ID | リスク | 影響度 | 発生確率 | 対策 |
|----|--------|--------|---------|------|
| R-020 | MIDIデバイスの互換性問題 | 中 | 中 | 複数のMIDIデバイスでテスト |
| R-021 | MIDIレイテンシ問題 | 高 | 中 | タイミング精度の検証、バッファ調整 |
| R-022 | 連符解析のパーサーバグ | 高 | 中 | 豊富なテストケース、エッジケース対応 |
| R-023 | 連符とタイの組み合わせの複雑性 | 中 | 中 | 段階的な実装、テストケース充実 |

### 8.2 未解決課題

| ID | 課題 | 担当 | 期限 |
|----|------|------|------|
| I-018 | midirクレートの詳細な使用方法確認 | 開発チーム | 2026-01-21 |
| I-019 | MIDI接続エラー時の自動再接続の実装検討 | 開発チーム | 2026-01-28 |
| I-020 | 連符のネスト深度の妥当性検証 | 開発チーム | 2026-02-04 |

---

## 9. 用語集

| 用語 | 定義 |
|------|------|
| MIDI | Musical Instrument Digital Interface。電子楽器間の通信規格 |
| MIDIノートオン | 音符の発音開始を指示するMIDIメッセージ |
| MIDIノートオフ | 音符の発音停止を指示するMIDIメッセージ |
| MIDIベロシティ | 音符の強さ（音量）を表す値（0-127） |
| MIDIチャンネル | MIDI通信のチャンネル（1-16） |
| 連符 | n個の音符を特定の音長に収める記法（3連符、5連符など） |
| 3連符 | 3つの音符を2つ分の音長に収める連符 |
| ベース音長 | 連符の基準となる音長 |

---

## 10. 参考リンク

### 10.1 MIDI仕様

#### MIDI規格
- **MIDI 1.0 Specification**: [MIDI Association](https://www.midi.org/specifications) - MIDI規格の公式仕様
- **MIDI Note Numbers**: [MIDI Note Number Chart](https://www.inspiredacoustics.com/en/MIDI_note_numbers_and_center_frequencies) - MIDIノートナンバーと周波数の対応表

### 10.2 Rust MIDIライブラリ

#### midir
- **GitHub Repository**: [midir](https://github.com/Boddlnagg/midir) - クロスプラットフォームMIDI I/O
- **docs.rs Documentation**: [midir](https://docs.rs/midir/) - API ドキュメント

### 10.3 連符の音楽理論

#### 連符の基礎
- **音楽理論 - 連符**: 連符は通常の音符の分割とは異なる分割方法
- **3連符**: 2つ分の音長を3等分する（例: 4分音符2つ分を3等分）
- **5連符**: 4つ分の音長を5等分する

### 10.4 MML連符の実装例

#### 他のMML実装
- **PPMCK MML Reference**: [MML Command Reference](http://ppmck.wikidot.com/mml-command-reference) - 連符の実装例
- **MML (Music Macro Language) - Wikipedia**: [日本語版](https://ja.wikipedia.org/wiki/Music_Macro_Language) - MMLの歴史と構文

---

## 11. 実装優先順位と段階的ロールアウト

### Phase 2.3.1（Week 1: 2026-01-14〜01-21）
1. **MIDIライブラリ調査** - midirクレートの使用方法確認
   - MIDIデバイス列挙
   - MIDIデバイス接続
   - MIDIメッセージ送信
2. **MIDIストリーミング基本実装** - `--midi-out` オプションの追加
   - CLIオプション追加
   - MIDIデバイス列挙機能
   - MIDIデバイス接続機能
   - MIDIノートオン/オフ送信

### Phase 2.3.2（Week 2: 2026-01-22〜01-28）
3. **連符構文実装** - トークナイザー・パーサー拡張
   - `{` `}` トークンの追加
   - `Tuplet` AST ノードの追加
   - パーサーでの連符解析
   - エラーハンドリング
4. **連符音長計算** - シンセサイザー実装
   - 連符音長の計算ロジック
   - ベース音長の処理
   - タイ記号との組み合わせ

### Phase 2.3.3（Week 3: 2026-01-29〜02-04）
5. **テストケース作成** - 豊富なテストケース
   - MIDIストリーミングのテスト
   - 連符のテスト（正常系、異常系、ネスト）
   - エッジケーステスト
6. **統合テスト** - MIDIと連符の組み合わせ
   - MIDIストリーミングで連符を演奏
   - パフォーマンステスト
   - 後方互換性テスト

---

## 12. テストケース概要

### 12.1 MIDIストリーミングのテストケース

| テストID | テストケース | 期待結果 |
|---------|-------------|---------|
| TC-031-001 | `--midi-list` | 利用可能なMIDIデバイス一覧が表示される |
| TC-031-002 | `--midi-out 0` | MIDIデバイス0にメッセージが送信される |
| TC-031-003 | `C4` をMIDI出力 | MIDIノート60が送信される |
| TC-031-004 | `V10` をMIDI出力 | MIDIベロシティ84が送信される |
| TC-031-005 | MIDIデバイス未接続 | エラー（MML-E015）が表示される |
| TC-031-006 | 無効なデバイスID | エラー（MML-E018）が表示される |
| TC-031-007 | Ctrl+Cで中断 | 全ノートオフメッセージが送信される |
| TC-031-008 | `--midi-channel 2` | MIDIチャンネル2でメッセージが送信される |
| TC-031-009 | `--midi-channel 10` | MIDIチャンネル10（ドラム）でメッセージが送信される |
| TC-031-010 | `--midi-channel 0` | エラー（MML-E024）が表示される |
| TC-031-011 | `--midi-channel 17` | エラー（MML-E024）が表示される |

### 12.2 連符のテストケース

| テストID | テストケース | 期待結果 |
|---------|-------------|---------|
| TC-032-001 | `{CDE}3` | 3連符が演奏される |
| TC-032-002 | `{CDEF}4` | 4連符が演奏される |
| TC-032-003 | `L8 {CDE}3` | 8分音符ベースの3連符が演奏される |
| TC-032-004 | `{CDE}3:2` | 2分音符に3音を収める連符が演奏される |
| TC-032-005 | `{CDR}3` | 休符を含む3連符が演奏される |
| TC-032-006 | `{C4&8 D E}3` | タイ記号を含む3連符が演奏される |
| TC-032-007 | `[{CDE}3]2` | ループと連符の組み合わせが演奏される |
| TC-032-008 | `{{CDE}3 FG}5` | ネストした連符が演奏される |
| TC-032-009 | `{CDE}` | エラー（MML-E021）が表示される |
| TC-032-010 | `{CDE}1` | エラー（MML-E022）が表示される |
| TC-032-011 | 6階層ネスト | エラー（MML-E023）が表示される |

---

## 13. スコープ変更の記録

### 13.1 REQ-CLI-001からの変更

REQ-CLI-001（v1.1.0）では以下のように定義されていました：

**1.4 スコープ > 対象外**:
- MIDI入出力

**変更内容**:
- Phase 2.3でMIDI出力を対象範囲に変更
- MIDI入力は引き続き対象外

**変更理由**:
- ユーザーからの要望により、外部MIDI機器との連携が必要となった
- MIDI出力のみであれば実装の複雑性が低く、Phase 2.3で対応可能

---

## 14. 変更履歴

| バージョン | 日付 | 変更内容 | 作成者 |
|-----------|------|----------|--------|
| 1.0.0 | 2026-01-14 | 初版作成 | req-writer |
| 1.0.1 | 2026-01-14 | MIDIチャンネル指定オプション（--midi-channel）追加、BR-104/MML-E024追加、テストケース追加 | req-writer |
