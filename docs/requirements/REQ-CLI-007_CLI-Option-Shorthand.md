# MML Synthesizer CLI オプションショートハンド拡充 要件定義書

## メタ情報

| 項目 | 内容 |
|------|------|
| ドキュメントID | REQ-CLI-007 |
| バージョン | 1.0.2 |
| ステータス | ドラフト |
| 作成日 | 2026-01-15 |
| 最終更新日 | 2026-01-15 |
| 作成者 | req-writer |
| 承認者 | - |
| 関連文書 | REQ-CLI-001_MML-Synthesizer.md (v1.1.0)<br>REQ-CLI-006_MIDI-Streaming-Tuplet.md (v1.0.1)<br>docs/memos/オプションのショートハンド充実.md |

---

## 1. プロジェクト概要

### 1.1 背景

sine-mml v2.3のMIDIストリーミング・連符機能実装完了後、以下の改善要望が発生しました：

1. **履歴の蓄積問題**: 開発中やテスト演奏時に履歴が蓄積し、有用な履歴が埋もれてしまう
2. **コマンド入力の煩雑さ**: 頻繁に使用するオプションの入力が長く、開発効率が低下
3. **オプション併用の明確化**: `--midi-out` と `--no-history`、`--midi-out` と `--loop-play` の併用動作を明確にする必要がある

これらの改善により、開発者体験（DX）を向上させ、より効率的なMML作成・テストワークフローを実現します。

### 1.2 目的

- `--no-history` オプションにより、テスト演奏時に履歴を汚さないことを可能にする
- `--no-history` にショートハンド `-N` を追加し、コマンド入力を簡略化する
- `--midi-out` と `--no-history` の併用動作を保証する
- `--midi-out` と `--loop-play` の併用動作を保証する

### 1.3 ゴール

| 目標 | 成功指標 |
|------|---------|
| 履歴スキップ機能の実装 | `--no-history` または `-N` で再生時に履歴が保存されない |
| ショートハンド対応 | `-N` で `--no-history` と同等に動作する |
| MIDI出力との併用 | `--midi-out` と `--no-history` が同時に指定可能 |
| ループ再生との併用 | `--midi-out` と `--loop-play` が同時に指定可能 |

### 1.4 スコープ

#### 対象範囲
- `--no-history` オプションの追加
- `-N` ショートハンドの追加
- `--midi-out` と `--no-history` の併用対応
- `--midi-out` と `--loop-play` の併用対応
- 既存ショートハンドとの整合性確認

#### 対象外
- 他のオプションへのショートハンド追加（将来検討）
- 履歴の選択的削除機能
- 履歴の自動クリーンアップ機能

---

## 2. ステークホルダー

### 2.1 ステークホルダー一覧

| 役割 | 担当者/部門 | 関心事 | 影響度 |
|------|------------|--------|--------|
| プロダクトオーナー | - | 開発者体験の向上、効率的なワークフロー | 高 |
| 開発チーム | - | CLI実装の簡潔性、テスト容易性 | 高 |
| エンドユーザー | 音楽制作者、開発者 | 履歴管理の柔軟性、コマンド入力の効率化 | 高 |

### 2.2 ユーザーペルソナ

#### ペルソナ1: 頻繁にテスト演奏する開発者
| 項目 | 内容 |
|------|------|
| 属性 | 20-40代、プログラマー、MML開発者 |
| 課題 | テスト演奏のたびに履歴が増え、有用な履歴が埋もれる |
| ニーズ | テスト演奏時は履歴を残さず、本番の作曲時のみ履歴に保存したい |
| 利用シーン | MML構文のテスト、一時的なフレーズ確認、デバッグ時の繰り返し再生 |

#### ペルソナ2: MIDI機器を使用する作曲者
| 項目 | 内容 |
|------|------|
| 属性 | 30-50代、音楽制作経験豊富 |
| 課題 | MIDI出力テスト時に履歴が蓄積する、ループ再生とMIDI出力を同時に使いたい |
| ニーズ | MIDI機器でのテスト時は履歴を残さない、ループ再生しながらMIDI出力したい |
| 利用シーン | 外部シンセサイザーでの音色確認、DAWとの連携テスト |

#### ペルソナ3: コマンドラインを効率的に使いたい開発者
| 項目 | 内容 |
|------|------|
| 属性 | 20-30代、CLI愛好者 |
| 課題 | オプションの入力が長く、繰り返しのコマンド入力が面倒 |
| ニーズ | ショートハンドで素早くコマンドを入力したい |
| 利用シーン | ターミナルでの連続作業、シェルスクリプトでの自動化 |

---

## 3. 機能要件

### 3.1 機能一覧

**※ REQ-CLI-001（F-001〜F-014）、REQ-CLI-002（F-015〜F-022）、REQ-CLI-003（F-023〜F-026）、REQ-CLI-004（F-027〜F-029）、REQ-CLI-005（F-030）、REQ-CLI-006（F-031〜F-032）との連番を維持**

| ID | 機能名 | 概要 | 優先度 | フェーズ | 備考 |
|----|--------|------|--------|---------|------|
| F-033 | 履歴スキップオプション | `--no-history` / `-N` で再生時に履歴をスキップ | 必須 | Phase 2.4 | **新規** |
| F-034 | MIDI出力と履歴スキップの併用 | `--midi-out` と `--no-history` の同時使用を保証 | 必須 | Phase 2.4 | **新規** |
| F-035 | MIDI出力とループ再生の併用 | `--midi-out` と `--loop-play` の同時使用を保証 | 必須 | Phase 2.4 | **新規** |
| F-036 | MIDI出力時のプログレスバー表示 | `--midi-out` 指定時（`--loop-play` なし）で通常再生と同様のプログレスバーを表示 | 必須 | Phase 2.4 | **新規** |

### 3.2 ユーザーストーリー

#### US-021: テスト演奏時に履歴を残さない
- **ユーザー**: 開発者
- **したいこと**: テスト演奏時に履歴を残さずに再生したい
- **理由**: テスト用の一時的なMMLで履歴を汚したくない
- **受け入れ基準**:
  - [ ] `sine-mml play "CDE" --no-history` で再生時に履歴が保存されない
  - [ ] `sine-mml play "CDE" -N` でも同様に履歴が保存されない
  - [ ] `sine-mml play -f test.mml --no-history` でファイル読み込み時も履歴が保存されない
  - [ ] `--no-history` を指定しない場合は従来通り履歴が保存される
  - [ ] 再生自体は正常に行われる（音が出る）
- **関連機能**: F-033

#### US-022: MIDI出力テスト時に履歴を残さない
- **ユーザー**: MIDI機器を使用する作曲者
- **したいこと**: MIDI出力テスト時に履歴を残さずに再生したい
- **理由**: MIDI機器の動作確認のための一時的な再生で履歴を汚したくない
- **受け入れ基準**:
  - [ ] `sine-mml play "CDE" --midi-out 0 --no-history` でMIDI出力され、履歴が保存されない
  - [ ] `sine-mml play "CDE" --midi-out 0 -N` でも同様に動作する
  - [ ] `--midi-out` のみ指定した場合は従来通り履歴が保存される
  - [ ] MIDI出力と内蔵シンセサイザー両方で再生する場合も履歴スキップが適用される
- **関連機能**: F-033, F-034

#### US-023: MIDI出力しながらループ再生したい
- **ユーザー**: MIDI機器を使用する作曲者
- **したいこと**: MIDI出力しながらループ再生したい
- **理由**: 外部シンセサイザーで繰り返し再生しながら音色を調整したい
- **受け入れ基準**:
  - [ ] `sine-mml play "CDE" --midi-out 0 --loop-play` でMIDI出力がループ再生される
  - [ ] Ctrl+C で正常に終了し、MIDIノートオフが送信される
  - [ ] ループ中も正確なテンポでMIDIメッセージが送信される
  - [ ] 内蔵シンセサイザーとMIDI出力を同時にループ再生できる
- **関連機能**: F-035

#### US-024: MIDI出力時に再生進行状況を確認したい
- **ユーザー**: MIDI機器を使用する作曲者
- **したいこと**: MIDI出力時に再生の進行状況を視覚的に確認したい
- **理由**: 外部シンセサイザーでの再生時に、あとどのくらいで演奏が終わるか把握したい
- **受け入れ基準**:
  - [ ] `sine-mml play "CDE" --midi-out 0` でプログレスバーが表示される
  - [ ] プログレスバーは再生進行に応じて0%→100%に更新される
  - [ ] `--loop-play` 指定時はプログレスバーを表示しない（終了位置がないため）
  - [ ] 内蔵シンセとMIDI出力の同時再生時も同一のプログレスバーを使用する
- **関連機能**: F-036

### 3.3 機能詳細

#### F-033: 履歴スキップオプション

**概要**: `--no-history` または `-N` オプションで再生時に履歴をスキップ

**入力**:
- MML文字列またはファイル
- `--no-history` または `-N` オプション

**出力**:
- 再生は行われるが、履歴には保存されない

**処理概要**:
1. CLIオプションを解析
2. `--no-history` フラグを確認
3. MMLを解析・再生
4. フラグが立っている場合、履歴保存処理をスキップ
5. 再生完了

**ビジネスルール**:
- BR-105: `--no-history` を指定した場合、履歴に保存しない
- BR-106: `-N` は `--no-history` と同等に動作する
- BR-107: `--no-history` を指定しない場合は従来通り履歴を保存する
- BR-108: 再生中のエラーが発生した場合、`--no-history` の指定有無に関わらず履歴には保存しない
- BR-109: `--note` オプションと `--no-history` が同時に指定された場合、`--note` の値は処理されないため、警告を表示して無視する

**制約事項**:
- 履歴IDでの再生（`--history-id`）と `--no-history` の組み合わせは許可する（単に再実行するが履歴には追加しない）

**技術実装のポイント**:

**1. CLIオプション追加**:
```rust
// src/cli/args.rs
#[derive(Args, Debug)]
pub struct PlayArgs {
    // 既存のフィールド...

    /// Do not save to history
    #[arg(long, short = 'N', default_value_t = false)]
    pub no_history: bool,
}
```

**注意**: clap では単一文字のショートハンドのみサポートされるため、`-N`（"No-history"の頭文字）を使用します。

**2. ハンドラーでの履歴スキップ**:
```rust
// src/cli/handlers.rs
pub async fn handle_play(args: PlayArgs) -> Result<(), Box<dyn std::error::Error>> {
    // MML取得処理...
    let mml = get_mml_content(&args)?;

    // 再生処理...
    play_mml(&mml, &args)?;

    // 履歴保存（--no-history が指定されていない場合のみ）
    if !args.no_history {
        save_to_history(&mml, args.note.as_deref())?;
    } else if args.note.is_some() {
        eprintln!("Warning: --note is ignored when --no-history is specified");
    }

    Ok(())
}
```

**受け入れテスト例**:
```rust
#[test]
fn test_no_history_flag() {
    let result = Cli::try_parse_from(&["sine-mml", "play", "CDE", "--no-history"]);
    assert!(result.is_ok());
    let args = match result.unwrap().command {
        Command::Play(args) => args,
        _ => panic!("Unexpected command"),
    };
    assert!(args.no_history);
}

#[test]
fn test_no_history_short_flag() {
    let result = Cli::try_parse_from(&["sine-mml", "play", "CDE", "-N"]);
    assert!(result.is_ok());
    let args = match result.unwrap().command {
        Command::Play(args) => args,
        _ => panic!("Unexpected command"),
    };
    assert!(args.no_history);
}

#[test]
fn test_no_history_default() {
    let result = Cli::try_parse_from(&["sine-mml", "play", "CDE"]);
    let args = match result.unwrap().command {
        Command::Play(args) => args,
        _ => panic!("Unexpected command"),
    };
    assert!(!args.no_history);
}
```

---

#### F-034: MIDI出力と履歴スキップの併用

**概要**: `--midi-out` と `--no-history` の同時使用を保証

**入力**:
- MML文字列またはファイル
- `--midi-out` オプション
- `--no-history` オプション

**出力**:
- MIDI出力が行われるが、履歴には保存されない

**処理概要**:
1. CLIオプションを解析
2. `--midi-out` と `--no-history` の両方のフラグを確認
3. MMLを解析
4. MIDI出力を実行
5. `--no-history` が指定されている場合、履歴保存をスキップ

**ビジネスルール**:
- BR-110: `--midi-out` と `--no-history` は独立して動作する
- BR-111: 両方指定した場合、MIDI出力は行われるが履歴には保存されない
- BR-112: `--waveform` と `--midi-out` と `--no-history` の3つを同時に指定可能

**技術実装のポイント**:

オプションは独立しているため、特別な実装は不要です。既存の `--midi-out` 処理と F-033 の `--no-history` 処理がそれぞれ独立して動作します。

**受け入れテスト例**:
```rust
#[cfg(feature = "midi-output")]
#[test]
fn test_midi_out_with_no_history() {
    let result = Cli::try_parse_from(&[
        "sine-mml", "play", "CDE",
        "--midi-out", "0",
        "--no-history"
    ]);
    assert!(result.is_ok());
    let args = match result.unwrap().command {
        Command::Play(args) => args,
        _ => panic!("Unexpected command"),
    };
    assert_eq!(args.midi_out, Some("0".to_string()));
    assert!(args.no_history);
}

#[cfg(feature = "midi-output")]
#[test]
fn test_midi_out_with_no_history_short() {
    let result = Cli::try_parse_from(&[
        "sine-mml", "play", "CDE",
        "--midi-out", "0",
        "-N"
    ]);
    assert!(result.is_ok());
    let args = match result.unwrap().command {
        Command::Play(args) => args,
        _ => panic!("Unexpected command"),
    };
    assert_eq!(args.midi_out, Some("0".to_string()));
    assert!(args.no_history);
}
```

---

#### F-035: MIDI出力とループ再生の併用

**概要**: `--midi-out` と `--loop-play` の同時使用を保証

**入力**:
- MML文字列またはファイル
- `--midi-out` オプション
- `--loop-play` オプション

**出力**:
- MIDI出力がループ再生される

**処理概要**:
1. CLIオプションを解析
2. `--midi-out` と `--loop-play` の両方のフラグを確認
3. MMLを解析
4. ループ再生モードでMIDI出力を実行
5. Ctrl+C でループを終了し、MIDIノートオフを送信

**ビジネスルール**:
- BR-113: `--midi-out` と `--loop-play` は独立して動作する
- BR-114: 両方指定した場合、MIDI出力がループ再生される
- BR-115: ループ終了時（Ctrl+C）は全MIDIノートオフメッセージを送信してクリーンアップ
- BR-116: `--waveform` と `--midi-out` と `--loop-play` の3つを同時に指定した場合、両方がループ再生される

**技術実装のポイント**:

オプションは独立しているため、特別な実装は不要です。既存の `--midi-out` 処理と `--loop-play` 処理がそれぞれ独立して動作します。ただし、MIDI出力のループ再生時にはノートオフの送信タイミングに注意が必要です。

**受け入れテスト例**:
```rust
#[cfg(feature = "midi-output")]
#[test]
fn test_midi_out_with_loop_play() {
    let result = Cli::try_parse_from(&[
        "sine-mml", "play", "CDE",
        "--midi-out", "0",
        "--loop-play"
    ]);
    assert!(result.is_ok());
    let args = match result.unwrap().command {
        Command::Play(args) => args,
        _ => panic!("Unexpected command"),
    };
    assert_eq!(args.midi_out, Some("0".to_string()));
    assert!(args.loop_play);
}

#[cfg(feature = "midi-output")]
#[test]
fn test_all_options_combined() {
    let result = Cli::try_parse_from(&[
        "sine-mml", "play", "CDE",
        "--midi-out", "0",
        "--loop-play",
        "--no-history"
    ]);
    assert!(result.is_ok());
    let args = match result.unwrap().command {
        Command::Play(args) => args,
        _ => panic!("Unexpected command"),
    };
    assert_eq!(args.midi_out, Some("0".to_string()));
    assert!(args.loop_play);
    assert!(args.no_history);
}
```

---

#### F-036: MIDI出力時のプログレスバー表示

**概要**: `--midi-out` 指定時（`--loop-play` なし）で通常再生と同様のプログレスバーアニメーションを表示

**入力**:
- MML文字列またはファイル
- `--midi-out` オプション（`--loop-play` なし）

**出力**:
- MIDI出力が行われる
- 通常の内蔵シンセ再生時と同様のプログレスバーアニメーションが表示される

**処理概要**:
1. CLIオプションを解析
2. `--midi-out` が指定され、`--loop-play` が指定されていないことを確認
3. MMLを解析し、全体の再生時間を計算
4. MIDI出力を開始
5. 再生位置に合わせてプログレスバーをアニメーション表示
6. 再生完了時にプログレスバーを100%表示して終了

**ビジネスルール**:
- BR-117: `--midi-out` 指定時（`--loop-play` なし）は通常再生と同様のプログレスバーを表示する
- BR-118: プログレスバーは再生位置（経過時間/全体時間）を反映する
- BR-119: `--loop-play` が指定されている場合はプログレスバーを表示しない（ループのため終了位置がない）
- BR-120: 内蔵シンセ + MIDI出力の同時使用時も、同一のプログレスバーを共有する

**制約事項**:
- プログレスバーはターミナル出力に依存するため、リダイレクト時は表示されない可能性がある

**受け入れテスト例**:
```rust
#[cfg(feature = "midi-output")]
#[test]
fn test_midi_out_shows_progress_bar() {
    // MIDI出力時（--loop-playなし）でプログレスバーが表示されることを確認
    // 実装は詳細設計で定義
}

#[cfg(feature = "midi-output")]
#[test]
fn test_midi_out_loop_no_progress_bar() {
    // MIDI出力 + --loop-play 時はプログレスバーが表示されないことを確認
}
```

---

## 4. 非機能要件

### 4.1 性能要件

| ID | 要件 | 目標値 | 測定方法 |
|----|------|--------|----------|
| NFR-P-019 | オプション解析のオーバーヘッド | 1ms以内 | ベンチマーク |
| NFR-P-020 | 履歴スキップによるパフォーマンス向上 | 履歴保存時間分（約5ms）の短縮 | ベンチマーク |

### 4.2 可用性要件

| ID | 要件 | 目標値 |
|----|------|--------|
| NFR-A-012 | オプション競合時のエラーハンドリング | 明確なエラーメッセージを表示 |
| NFR-A-013 | 警告メッセージの表示 | `--note` と `--no-history` 併用時に警告を表示 |

### 4.3 セキュリティ要件

| ID | 要件 | 詳細 |
|----|------|------|
| NFR-S-015 | 履歴スキップの信頼性 | `--no-history` 指定時は確実に履歴に保存しない |

### 4.4 ユーザビリティ要件

| ID | 要件 | 詳細 |
|----|------|------|
| NFR-U-019 | ショートハンドの直感性 | `-N` は "No-history" を連想させる |
| NFR-U-020 | ヘルプメッセージの明確性 | `--help` で `--no-history` / `-N` の説明を表示 |
| NFR-U-021 | オプション組み合わせの自由度 | 任意のオプションを自由に組み合わせ可能 |

### 4.5 保守性要件

| ID | 要件 | 詳細 |
|----|------|------|
| NFR-M-019 | 履歴スキップのテスト | 正常系、異常系を網羅 |
| NFR-M-020 | オプション併用のテスト | 全ての組み合わせパターンをテスト |
| NFR-M-021 | 後方互換性 | 既存のコマンドが正常に動作 |

---

## 5. 制約条件

### 5.1 技術的制約

| 制約 | 詳細 | 理由 |
|------|------|------|
| ショートハンドは単一文字 | clap の制約により、ショートハンドは1文字のみ | clapライブラリの仕様 |
| `-h` は使用不可 | ヘルプ用に予約済み | clap のデフォルト動作 |
| `-N` を使用 | "No-history" の頭文字で直感的、かつ未使用 | 可読性と一貫性 |

### 5.2 ビジネス制約

| 制約 | 詳細 |
|------|------|
| 予算 | オープンソース（無償） |
| スケジュール | Phase 2.4: 1週間 |
| リソース | 個人開発（1名） |

### 5.3 法規制・コンプライアンス

| 要件 | 詳細 |
|------|------|
| ライセンス | MIT License（変更なし） |
| 後方互換性 | 既存のコマンドが正常に動作することを保証 |

---

## 6. 外部インターフェース

### 6.1 既存機能への影響

| 機能ID | 機能名 | 影響内容 | 対応 |
|--------|--------|----------|------|
| F-007 | 履歴機能 | **拡張** | 履歴保存のスキップオプションを追加 |
| F-031 | MIDIストリーミング | **確認** | `--no-history` との併用を保証 |
| F-002 | リアルタイム再生 | **確認** | `--loop-play` との併用を保証（既存） |

### 6.2 CLIオプションの追加

#### 新規オプション
```bash
# 履歴をスキップして再生
sine-mml play "T120 L4 O4 CDEFGAB" --no-history
sine-mml play "T120 L4 O4 CDEFGAB" -N

# ファイル読み込み + 履歴スキップ
sine-mml play -f test.mml --no-history
sine-mml play -f test.mml -N

# MIDI出力 + 履歴スキップ
sine-mml play "CDEFGAB" --midi-out 0 --no-history
sine-mml play "CDEFGAB" --midi-out 0 -N

# MIDI出力 + ループ再生
sine-mml play "CDEFGAB" --midi-out 0 --loop-play

# 全オプション併用
sine-mml play "CDEFGAB" --midi-out 0 --loop-play --no-history
sine-mml play "CDEFGAB" --midi-out 0 --loop-play -N
```

### 6.3 既存ショートハンドとの整合性

| オプション | ショートハンド | 用途 |
|-----------|--------------|------|
| `--file` | `-f` | ファイル読み込み |
| `--waveform` | `-w` | 波形選択 |
| `--volume` | `-v` | 音量設定 |
| `--output` | `-o` | エクスポート出力先 |
| `--no-history` | `-N` | 履歴スキップ **新規** |

---

## 7. 前提条件と依存関係

### 7.1 前提条件

- REQ-CLI-001の全機能が実装済み（v1.0.0リリース済み）
- REQ-CLI-002の全機能が実装済み（v2.0.0リリース済み）
- REQ-CLI-003の全機能が実装済み（v2.1.0リリース済み）
- REQ-CLI-004の全機能が実装済み（v2.1.0リリース済み）
- REQ-CLI-005の全機能が実装済み（v2.2.0リリース済み）
- REQ-CLI-006の全機能が実装済み（v2.3.0リリース済み）
- REQ-CLI-006のMIDI出力機能（F-031）が実装済み（F-034、F-035の前提条件）

### 7.2 依存関係

| 依存先 | 内容 | 影響 |
|--------|------|------|
| clap | CLIオプション解析 | オプション追加 |
| 履歴機能 | 履歴保存処理 | 条件分岐追加 |

### 7.3 新規依存クレート

なし（既存のクレートで対応可能）

---

## 8. リスクと課題

### 8.1 リスク一覧

| ID | リスク | 影響度 | 発生確率 | 対策 |
|----|--------|--------|---------|------|
| R-024 | ショートハンドの衝突 | 低 | 低 | 既存ショートハンドの一覧を確認済み |
| R-025 | オプション併用時の予期しない動作 | 中 | 低 | 全組み合わせパターンのテストを実施 |
| R-026 | ユーザーの混乱（`-N` の意味） | 低 | 低 | ヘルプメッセージで明確に説明 |

### 8.2 未解決課題

| ID | 課題 | 担当 | 期限 |
|----|------|------|------|
| I-021 | 他のオプションへのショートハンド追加の検討 | 開発チーム | Phase 3.0で検討 |
| I-022 | 履歴の自動クリーンアップ機能の検討 | 開発チーム | 将来バージョンで検討 |

---

## 9. 用語集

| 用語 | 定義 |
|------|------|
| ショートハンド | CLIオプションの短縮形（例: `--file` → `-f`） |
| 履歴スキップ | 再生時に履歴に保存しない機能 |
| ループ再生 | 曲を繰り返し再生する機能 |
| MIDI出力 | 外部MIDI機器にMIDIメッセージを送信する機能 |

---

## 10. 参考リンク

### 10.1 Clap CLI ライブラリ

#### 公式ドキュメント
- **Clap Documentation**: [docs.rs/clap](https://docs.rs/clap/) - CLIオプション定義の公式ドキュメント
- **Clap GitHub**: [clap-rs/clap](https://github.com/clap-rs/clap) - ソースコードと例

### 10.2 CLIデザインのベストプラクティス

#### ガイドライン
- **Command Line Interface Guidelines**: [clig.dev](https://clig.dev/) - CLIデザインのベストプラクティス
- **GNU Coding Standards - Command-Line Interfaces**: [GNU Standards](https://www.gnu.org/prep/standards/html_node/Command_002dLine-Interfaces.html)

---

## 11. 実装優先順位と段階的ロールアウト

### Phase 2.4.1（Day 1-2: 2026-01-15〜01-16）
1. **CLIオプション追加** - `--no-history` / `-N` の実装
   - `PlayArgs` 構造体に `no_history` フィールド追加
   - clap のオプション定義
   - ヘルプメッセージの追加

### Phase 2.4.2（Day 3-4: 2026-01-17〜01-18）
2. **履歴スキップロジック** - ハンドラーの修正
   - `handle_play` での条件分岐追加
   - 警告メッセージの実装（`--note` との併用時）

### Phase 2.4.3（Day 5: 2026-01-19）
3. **テストケース作成** - 豊富なテストケース
   - 正常系テスト
   - 異常系テスト
   - オプション併用テスト
   - 後方互換性テスト

---

## 12. テストケース概要

### 12.1 履歴スキップのテストケース

| テストID | テストケース | 期待結果 |
|---------|-------------|---------|
| TC-033-001 | `play "CDE" --no-history` | 再生され、履歴に保存されない |
| TC-033-002 | `play "CDE" -N` | 再生され、履歴に保存されない |
| TC-033-003 | `play "CDE"` | 再生され、履歴に保存される（従来通り） |
| TC-033-004 | `play -f test.mml --no-history` | ファイル読み込み、履歴に保存されない |
| TC-033-005 | `play --history-id 1 --no-history` | 履歴から再生され、新たな履歴は追加されない |
| TC-033-006 | `play "CDE" --no-history --note "Test"` | 再生され、警告が表示され、履歴に保存されない |

### 12.2 MIDI出力との併用テストケース

| テストID | テストケース | 期待結果 |
|---------|-------------|---------|
| TC-034-001 | `play "CDE" --midi-out 0 --no-history` | MIDI出力され、履歴に保存されない |
| TC-034-002 | `play "CDE" --midi-out 0 -N` | MIDI出力され、履歴に保存されない |
| TC-034-003 | `play "CDE" --midi-out 0` | MIDI出力され、履歴に保存される（従来通り） |
| TC-034-004 | `play "CDE" -w sine --midi-out 0 --no-history` | 両方で再生され、履歴に保存されない |

### 12.3 ループ再生との併用テストケース

| テストID | テストケース | 期待結果 |
|---------|-------------|---------|
| TC-035-001 | `play "CDE" --midi-out 0 --loop-play` | MIDI出力がループ再生される |
| TC-035-002 | `play "CDE" --midi-out 0 --loop-play --no-history` | MIDI出力がループ再生され、履歴に保存されない |
| TC-035-003 | `play "CDE" -w sine --midi-out 0 --loop-play` | 両方がループ再生される |

### 12.4 MIDI出力時のプログレスバー表示テストケース

| テストID | テストケース | 期待結果 |
|---------|-------------|---------|
| TC-036-001 | `play "CDE" --midi-out 0` | MIDI出力され、プログレスバーが表示される |
| TC-036-002 | `play "CDE" --midi-out 0 --loop-play` | MIDI出力がループ再生され、プログレスバーは表示されない |
| TC-036-003 | `play "CDE" -w sine --midi-out 0` | 内蔵シンセ + MIDI出力され、プログレスバーが表示される |
| TC-036-004 | `play "CDE" --midi-out 0 --no-history` | MIDI出力され、プログレスバー表示、履歴保存なし |
| TC-036-005 | プログレスバーが0%→100%に更新される | 再生進行に応じてプログレスバーが更新される |

### 12.5 CLIオプション解析のテストケース

| テストID | テストケース | 期待結果 |
|---------|-------------|---------|
| TC-033-007 | `--no-history` フラグのパース | `no_history` が `true` |
| TC-033-008 | `-N` フラグのパース | `no_history` が `true` |
| TC-033-009 | デフォルト値 | `no_history` が `false` |
| TC-033-010 | `--help` | `--no-history` / `-N` の説明が表示される |

---

## 13. 変更履歴

| バージョン | 日付 | 変更内容 | 作成者 |
|-----------|------|----------|--------|
| 1.0.0 | 2026-01-15 | 初版作成 | req-writer |
| 1.0.1 | 2026-01-15 | レビュー指摘対応: ショートハンド`-N`に統一、BR-108/BR-109明確化、MIDI前提条件追記 | req-writer |
| 1.0.2 | 2026-01-15 | F-036（MIDI出力時のプログレスバー表示）追加、US-024追加、BR-117〜BR-120追加、TC-036-001〜005追加 | req-writer |
