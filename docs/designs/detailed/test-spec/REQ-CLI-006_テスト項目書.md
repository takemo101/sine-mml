# テスト項目書: REQ-CLI-006 MIDI Streaming & Tuplet

## メタ情報

| 項目 | 内容 |
|------|------|
| ドキュメントID | TEST-CLI-006 |
| 対応設計書 | BASIC-CLI-006_MIDI-Streaming-Tuplet.md (v1.0.0) |
| 作成日 | 2026-01-14 |
| ステータス | ドラフト |
| 対象バージョン | sine-mml v3.0 |

---

## 1. テスト方針

### 1.1 テストレベル

| レベル | 対象 | カバレッジ目標 |
|--------|------|---------------|
| Unit | パーサー、MIDI変換、音長計算 | 95%以上 |
| Integration | CLI引数処理、MIDIデバイス接続 | 90%以上 |
| E2E | CLIコマンド実行、MIDI出力確認 | 主要パス100% |

### 1.2 テスト優先度

| 優先度 | 説明 | 実装タイミング |
|--------|------|---------------|
| P0 | クリティカルパス（必須機能） | Sprint 1 Week 1 |
| P1 | 重要機能（エラーハンドリング） | Sprint 1 Week 2 |
| P2 | エッジケース（境界値） | Sprint 2 |

### 1.3 テスト環境

- **OS**: macOS, Linux (CI環境)
- **Rust**: 1.70+
- **テストフレームワーク**: 
  - ユニットテスト: Rust標準 `#[test]`
  - E2Eテスト: `assert_cmd`, `predicates`, `tempfile`
- **MIDIテスト**: モックデバイス（CI環境では実デバイス不要）

---

## 2. テスト対象機能一覧

| 機能ID | 機能名 | 概要 | 優先度 |
|--------|--------|------|--------|
| F-031 | MIDIストリーミング | `--midi-out` でMIDIデバイスにリアルタイム送信 | P0 |
| F-032 | 連符（n連符） | `{...}n` で連符を表現 | P0 |

---

## 3. ユニットテスト項目

### 3.1 F-031: MIDIストリーミング

#### TC-031-U-001: MML音程→MIDIノートナンバー変換

| 項目 | 内容 |
|------|------|
| 優先度 | P0 |
| テスト対象 | `mml_to_midi_note()` |
| 前提条件 | - |
| 入力 | `Pitch::C, None, 4` |
| 期待結果 | `60` (C4 = Middle C) |

**テストコード雛形:**
```rust
#[test]
fn test_mml_to_midi_note_c4() {
    let note = mml_to_midi_note(Pitch::C, None, 4);
    assert_eq!(note, 60);
}

#[test]
fn test_mml_to_midi_note_c_sharp_4() {
    let note = mml_to_midi_note(Pitch::C, Some(Accidental::Sharp), 4);
    assert_eq!(note, 61);
}

#[test]
fn test_mml_to_midi_note_a4() {
    let note = mml_to_midi_note(Pitch::A, None, 4);
    assert_eq!(note, 69); // A4 = 440Hz
}
```

---

#### TC-031-U-002: MML音量→MIDIベロシティ変換

| 項目 | 内容 |
|------|------|
| 優先度 | P0 |
| テスト対象 | `mml_volume_to_velocity()` |
| 前提条件 | - |
| 入力 | `0`, `10`, `15` |
| 期待結果 | `0`, `84`, `127` |

**テストコード雛形:**
```rust
#[test]
fn test_volume_to_velocity() {
    assert_eq!(mml_volume_to_velocity(0), 0);
    assert_eq!(mml_volume_to_velocity(10), 84);
    assert_eq!(mml_volume_to_velocity(15), 127);
}
```

---

#### TC-031-U-003: MIDIチャンネル範囲検証

| 項目 | 内容 |
|------|------|
| 優先度 | P0 |
| テスト対象 | `validate_midi_channel()` |
| 前提条件 | - |
| 入力 | `0`, `1`, `16`, `17` |
| 期待結果 | `Err`, `Ok(1)`, `Ok(16)`, `Err` |

**テストコード雛形:**
```rust
#[test]
fn test_midi_channel_validation() {
    assert!(validate_midi_channel(0).is_err());
    assert_eq!(validate_midi_channel(1), Ok(1));
    assert_eq!(validate_midi_channel(16), Ok(16));
    assert!(validate_midi_channel(17).is_err());
}
```

---

#### TC-031-U-004: MIDIノートオンメッセージ構築

| 項目 | 内容 |
|------|------|
| 優先度 | P0 |
| テスト対象 | `build_note_on_message()` |
| 前提条件 | - |
| 入力 | `channel=1, note=60, velocity=100` |
| 期待結果 | `[0x90, 60, 100]` |

**テストコード雛形:**
```rust
#[test]
fn test_note_on_message() {
    let msg = build_note_on_message(1, 60, 100);
    assert_eq!(msg, [0x90, 60, 100]);
}

#[test]
fn test_note_on_message_channel_10() {
    let msg = build_note_on_message(10, 36, 127);
    assert_eq!(msg, [0x99, 36, 127]); // 0x90 | (10-1)
}
```

---

#### TC-031-U-005: MIDIノートオフメッセージ構築

| 項目 | 内容 |
|------|------|
| 優先度 | P0 |
| テスト対象 | `build_note_off_message()` |
| 前提条件 | - |
| 入力 | `channel=1, note=60` |
| 期待結果 | `[0x80, 60, 0]` |

**テストコード雛形:**
```rust
#[test]
fn test_note_off_message() {
    let msg = build_note_off_message(1, 60);
    assert_eq!(msg, [0x80, 60, 0]);
}
```

---

#### TC-031-U-006: 全ノートオフメッセージ構築

| 項目 | 内容 |
|------|------|
| 優先度 | P1 |
| テスト対象 | `build_all_notes_off_message()` |
| 前提条件 | - |
| 入力 | `channel=1` |
| 期待結果 | `[0xB0, 123, 0]` |

**テストコード雛形:**
```rust
#[test]
fn test_all_notes_off_message() {
    let msg = build_all_notes_off_message(1);
    assert_eq!(msg, [0xB0, 123, 0]); // CC 123 = All Notes Off
}
```

---

### 3.2 F-032: 連符（n連符）

#### TC-032-U-001: 基本連符構文の解析（3連符）

| 項目 | 内容 |
|------|------|
| 優先度 | P0 |
| テスト対象 | `Parser::parse_tuplet()` |
| 前提条件 | トークナイザーが `{CDE}3` をトークン列に変換済み |
| 入力 | `{CDE}3` |
| 期待結果 | `Command::Tuplet { notes: [C,D,E], count: 3, base_duration: None }` |

**テストコード雛形:**
```rust
#[test]
fn test_parse_basic_tuplet() {
    let input = "{CDE}3";
    let mml = parse(input).unwrap();
    
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Tuplet { notes, count, base_duration } => {
            assert_eq!(notes.len(), 3);
            assert_eq!(*count, 3);
            assert_eq!(*base_duration, None);
        }
        _ => panic!("Expected Tuplet command"),
    }
}
```

---

#### TC-032-U-002: ベース音長指定の連符解析

| 項目 | 内容 |
|------|------|
| 優先度 | P0 |
| テスト対象 | `Parser::parse_tuplet()` |
| 前提条件 | - |
| 入力 | `{CDE}3:2` |
| 期待結果 | `Command::Tuplet { notes: [C,D,E], count: 3, base_duration: Some(2) }` |

**テストコード雛形:**
```rust
#[test]
fn test_parse_tuplet_with_base_duration() {
    let input = "{CDE}3:2";
    let mml = parse(input).unwrap();
    
    match &mml.commands[0] {
        Command::Tuplet { notes, count, base_duration } => {
            assert_eq!(notes.len(), 3);
            assert_eq!(*count, 3);
            assert_eq!(*base_duration, Some(2));
        }
        _ => panic!("Expected Tuplet command"),
    }
}
```

---

#### TC-032-U-003: 連符数未指定エラー

| 項目 | 内容 |
|------|------|
| 優先度 | P0 |
| テスト対象 | `Parser::parse_tuplet()` |
| 前提条件 | - |
| 入力 | `{CDE}` |
| 期待結果 | `Err(MmlError::TupletCountMissing)` (MML-E021) |

**テストコード雛形:**
```rust
#[test]
fn test_tuplet_count_missing_error() {
    let input = "{CDE}";
    let result = parse(input);
    
    assert!(matches!(result, Err(MmlError::TupletCountMissing { .. })));
}
```

---

#### TC-032-U-004: 無効な連符数エラー（1以下）

| 項目 | 内容 |
|------|------|
| 優先度 | P0 |
| テスト対象 | `Parser::parse_tuplet()` |
| 前提条件 | - |
| 入力 | `{CDE}1`, `{CDE}0` |
| 期待結果 | `Err(MmlError::InvalidTupletCount)` (MML-E022) |

**テストコード雛形:**
```rust
#[test]
fn test_invalid_tuplet_count_one() {
    let input = "{CDE}1";
    let result = parse(input);
    
    assert!(matches!(result, Err(MmlError::InvalidTupletCount { .. })));
}

#[test]
fn test_invalid_tuplet_count_zero() {
    let input = "{CDE}0";
    let result = parse(input);
    
    assert!(matches!(result, Err(MmlError::InvalidTupletCount { .. })));
}
```

---

#### TC-032-U-005: 連符ネスト深度超過エラー

| 項目 | 内容 |
|------|------|
| 優先度 | P1 |
| テスト対象 | `Parser::parse_tuplet()` |
| 前提条件 | - |
| 入力 | 6階層ネスト `{{{{{{C}2}2}2}2}2}2` |
| 期待結果 | `Err(MmlError::TupletNestTooDeep)` (MML-E023) |

**テストコード雛形:**
```rust
#[test]
fn test_tuplet_nest_too_deep() {
    let input = "{{{{{{C}2}2}2}2}2}2"; // 6階層
    let result = parse(input);
    
    assert!(matches!(result, Err(MmlError::TupletNestTooDeep { .. })));
}
```

---

#### TC-032-U-006: 閉じ括弧なしエラー

| 項目 | 内容 |
|------|------|
| 優先度 | P0 |
| テスト対象 | `Parser::parse_tuplet()` |
| 前提条件 | - |
| 入力 | `{CDE` |
| 期待結果 | `Err(MmlError::UnclosedTuplet)` (MML-E020) |

**テストコード雛形:**
```rust
#[test]
fn test_unclosed_tuplet() {
    let input = "{CDE";
    let result = parse(input);
    
    assert!(matches!(result, Err(MmlError::UnclosedTuplet { .. })));
}
```

---

#### TC-032-U-007: 連符音長計算（3連符）

| 項目 | 内容 |
|------|------|
| 優先度 | P0 |
| テスト対象 | `Tuplet::duration_in_seconds()` |
| 前提条件 | BPM=120, デフォルト音長=4 |
| 入力 | `{CDE}3` |
| 期待結果 | 各音 `0.167秒`、合計 `0.5秒` |

**テストコード雛形:**
```rust
#[test]
fn test_tuplet_duration_triplet() {
    let tuplet = Tuplet {
        notes: vec![Note::new(Pitch::C), Note::new(Pitch::D), Note::new(Pitch::E)],
        count: 3,
        base_duration: None,
    };
    
    let durations = tuplet.note_durations_in_seconds(120, 4);
    
    // 各音: 0.5 / 3 = 0.167秒
    for d in &durations {
        assert!((d - 0.167).abs() < 0.001);
    }
    
    // 合計: 0.5秒
    let total: f32 = durations.iter().sum();
    assert!((total - 0.5).abs() < 0.001);
}
```

---

#### TC-032-U-008: ネストした連符の音長計算

| 項目 | 内容 |
|------|------|
| 優先度 | P1 |
| テスト対象 | `Tuplet::duration_in_seconds()` |
| 前提条件 | BPM=120, デフォルト音長=4 |
| 入力 | `{{CDE}3 FG}5` |
| 期待結果 | 内側3連符: `0.033秒/音`、外側FG: `0.1秒/音` |

**テストコード雛形:**
```rust
#[test]
fn test_nested_tuplet_duration() {
    let input = "{{CDE}3 FG}5";
    let mml = parse(input).unwrap();
    
    let total_duration = mml.total_duration_in_seconds(120, 4);
    
    // 外側5連符: 0.5 / 5 = 0.1秒/音
    // 内側3連符（1音分）: 0.1 / 3 = 0.033秒/音
    // 合計: 0.1 (inner) + 0.1 + 0.1 = 0.3秒
    assert!((total_duration - 0.3).abs() < 0.001);
}
```

---

#### TC-032-U-009: 休符を含む連符

| 項目 | 内容 |
|------|------|
| 優先度 | P0 |
| テスト対象 | `Parser::parse_tuplet()` |
| 前提条件 | - |
| 入力 | `{CDR}3` |
| 期待結果 | `Command::Tuplet { notes: [C,D,Rest], count: 3, base_duration: None }` |

**テストコード雛形:**
```rust
#[test]
fn test_tuplet_with_rest() {
    let input = "{CDR}3";
    let mml = parse(input).unwrap();
    
    match &mml.commands[0] {
        Command::Tuplet { notes, count, .. } => {
            assert_eq!(notes.len(), 3);
            assert!(matches!(notes[2], Command::Rest { .. }));
            assert_eq!(*count, 3);
        }
        _ => panic!("Expected Tuplet command"),
    }
}
```

---

## 4. 統合テスト項目

### 4.1 F-031: MIDIストリーミング

#### TC-031-I-001: --midi-list オプション

| 項目 | 内容 |
|------|------|
| 優先度 | P0 |
| テスト対象 | CLI `--midi-list` オプション |
| 前提条件 | - |
| 入力 | `sine-mml play "C" --midi-list` |
| 期待結果 | MIDIデバイス一覧が表示される、または「デバイスなし」メッセージ |

---

#### TC-031-I-002: --midi-channel オプション範囲検証

| 項目 | 内容 |
|------|------|
| 優先度 | P0 |
| テスト対象 | CLI `--midi-channel` オプション |
| 前提条件 | - |
| 入力 | `sine-mml play "C" --midi-out 0 --midi-channel 0` |
| 期待結果 | エラー `[MML-E024] 無効なMIDIチャンネルです（1-16を指定してください）: 0` |

---

### 4.2 F-032: 連符

#### TC-032-I-001: 連符とループの組み合わせ

| 項目 | 内容 |
|------|------|
| 優先度 | P0 |
| テスト対象 | パーサー統合 |
| 前提条件 | - |
| 入力 | `[{CDE}3]2` |
| 期待結果 | 3連符が2回繰り返し再生される |

---

#### TC-032-I-002: 連符とタイ記号の組み合わせ

| 項目 | 内容 |
|------|------|
| 優先度 | P1 |
| テスト対象 | パーサー統合 |
| 前提条件 | - |
| 入力 | `{C4&8 D E}3` |
| 期待結果 | タイ記号を含む連符が正しく演奏される |

---

## 5. E2Eテスト項目

### 5.1 F-031: MIDIストリーミング

#### TC-031-E-001: MIDIデバイスなしでのエラーメッセージ

| 項目 | 内容 |
|------|------|
| 優先度 | P0 |
| テスト対象 | CLI E2E |
| 前提条件 | MIDIデバイスが接続されていない |
| 入力 | `sine-mml play "CDEF" --midi-out 0` |
| 期待結果 | 終了コード1、stderr に `[MML-E015] MIDIデバイスが見つかりません` |

**テストコード雛形:**
```rust
#[test]
fn test_midi_no_device_error() {
    let mut cmd = Command::cargo_bin("sine-mml").unwrap();
    cmd.args(["play", "CDEF", "--midi-out", "99"]);
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("[MML-E018]"));
}
```

---

### 5.2 F-032: 連符

#### TC-032-E-001: 基本連符の再生

| 項目 | 内容 |
|------|------|
| 優先度 | P0 |
| テスト対象 | CLI E2E |
| 前提条件 | - |
| 入力 | `sine-mml play "{CDE}3"` |
| 期待結果 | 終了コード0、3連符が再生される |

**テストコード雛形:**
```rust
#[test]
fn test_tuplet_playback() {
    let mut cmd = Command::cargo_bin("sine-mml").unwrap();
    cmd.args(["play", "{CDE}3"]);
    
    cmd.assert()
        .success();
}
```

---

#### TC-032-E-002: 連符数エラーメッセージ

| 項目 | 内容 |
|------|------|
| 優先度 | P0 |
| テスト対象 | CLI E2E |
| 前提条件 | - |
| 入力 | `sine-mml play "{CDE}1"` |
| 期待結果 | 終了コード1、stderr に `[MML-E022]` |

**テストコード雛形:**
```rust
#[test]
fn test_invalid_tuplet_count_error() {
    let mut cmd = Command::cargo_bin("sine-mml").unwrap();
    cmd.args(["play", "{CDE}1"]);
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("[MML-E022]"));
}
```

---

#### TC-032-E-003: 連符ネスト深度超過エラーメッセージ

| 項目 | 内容 |
|------|------|
| 優先度 | P1 |
| テスト対象 | CLI E2E |
| 前提条件 | - |
| 入力 | `sine-mml play "{{{{{{C}2}2}2}2}2}2"` |
| 期待結果 | 終了コード1、stderr に `[MML-E023]` |

**テストコード雛形:**
```rust
#[test]
fn test_tuplet_nest_too_deep_error() {
    let mut cmd = Command::cargo_bin("sine-mml").unwrap();
    cmd.args(["play", "{{{{{{C}2}2}2}2}2}2"]);
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("[MML-E023]"));
}
```

---

## 6. テストケースサマリー

| 機能ID | ユニットテスト | 統合テスト | E2Eテスト | 合計 |
|--------|--------------|-----------|----------|------|
| F-031 | 6 | 2 | 1 | 9 |
| F-032 | 9 | 2 | 3 | 14 |
| **合計** | **15** | **4** | **4** | **23** |

---

## 7. エラーコードとテストケースの対応

| エラーコード | 説明 | テストケースID |
|-------------|------|---------------|
| MML-E015 | MIDIデバイスが見つからない | TC-031-E-001 |
| MML-E016 | MIDIデバイス接続エラー | - (実デバイス依存) |
| MML-E017 | MIDIメッセージ送信エラー | - (実デバイス依存) |
| MML-E018 | 無効なMIDIデバイスID | TC-031-E-001 |
| MML-E019 | MIDIデバイス切断 | - (実デバイス依存) |
| MML-E020 | 連符の閉じ括弧がない | TC-032-U-006 |
| MML-E021 | 連符数が指定されていない | TC-032-U-003 |
| MML-E022 | 無効な連符数 | TC-032-U-004, TC-032-E-002 |
| MML-E023 | 連符のネスト深度超過 | TC-032-U-005, TC-032-E-003 |
| MML-E024 | 無効なMIDIチャンネル | TC-031-U-003, TC-031-I-002 |

---

## 変更履歴

| 日付 | バージョン | 変更内容 | 担当者 |
|:---|:---|:---|:---|
| 2026-01-14 | 1.0.0 | 初版作成 | detailed-design-writer |
