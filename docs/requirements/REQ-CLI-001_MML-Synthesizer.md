# MML Synthesizer CLI アプリケーション 要件定義書

## メタ情報

| 項目 | 内容 |
|------|------|
| ドキュメントID | REQ-CLI-001 |
| バージョン | 1.1.0 |
| ステータス | ドラフト |
| 作成日 | 2026-01-10 |
| 最終更新日 | 2026-01-10 |
| 作成者 | Antigravity |
| 承認者 | - |

---

## 1. プロジェクト概要

### 1.1 背景
Music Macro Language (MML) は、BASIC言語などで音楽を文字列で記述するために広く使われてきた言語です。
本プロジェクトでは、思いついたメロディやアイディアをMMLで手軽に出力できるCLIアプリケーションを開発します。

### 1.2 目的
- MML記法でサイン波などの波形を即座に演奏できる環境を提供
- メロディのアイディアを素早く試聴・記録できるツールの実現
- シンプルなCLI操作で音楽制作のワークフローを効率化

### 1.3 ゴール
- コマンドライン引数にMMLを入力すると即座に演奏が開始される
- 過去のメロディを履歴として保存し、再生・WAV出力が可能
- 複数の波形（サイン波、ノコギリ波、矩形波）で演奏可能

### 1.4 スコープ

#### 対象範囲
- MMLパーサーの実装
- リアルタイム音声合成・再生（モノフォニック）
- 履歴管理機能（SQLite）
- WAVファイル出力
- CLI インターフェース

#### 対象外
- ポリフォニック（和音同時発音）対応
- GUI インターフェース
- ネットワーク経由の共有機能
- MIDI入出力

---

## 2. ステークホルダー

### 2.1 ステークホルダー一覧

| 役割 | 担当者/部門 | 関心事 | 影響度 |
|------|------------|--------|--------|
| プロダクトオーナー | - | 使いやすいCLIツール | 高 |
| 開発チーム | - | 実装の容易性、保守性 | 高 |
| エンドユーザー | 音楽制作者、趣味プログラマー | 手軽なメロディ試聴 | 高 |

### 2.2 ユーザーペルソナ

#### ペルソナ1: 趣味で作曲する開発者
| 項目 | 内容 |
|------|------|
| 属性 | 20-40代、プログラミング経験あり |
| 課題 | 思いついたメロディをすぐに確認したいが、DAWを起動するのが手間 |
| ニーズ | コマンドラインから即座にメロディを試聴・記録 |
| 利用シーン | コーディング中のアイディア出し、通勤中のメモ確認 |

---

## 3. 機能要件

### 3.1 機能一覧

| ID | 機能名 | 概要 | 優先度 | フェーズ |
|----|--------|------|--------|---------|
| F-001 | MMLパーサー | MML文字列を解析し音符データに変換 | 必須 | Phase 1 |
| F-002 | リアルタイム再生 | 解析したMMLをリアルタイムで音声出力 | 必須 | Phase 1 |
| F-003 | 波形選択 | サイン波、ノコギリ波、矩形波の選択 | 必須 | Phase 1 |
| F-004 | 音量調節 | 音量をオプションで調節可能 | 必須 | Phase 1 |
| F-005 | BPM調節 | テンポをオプションで調節可能 | 必須 | Phase 1 |
| F-006 | ループ再生 | 演奏を繰り返し再生 | 必須 | Phase 1 |
| F-007 | 履歴保存 | MML入力を自動的にデータベースに保存 | 必須 | Phase 1 |
| F-008 | 履歴一覧表示 | 保存された履歴を作成日付とともに一覧表示 | 必須 | Phase 1 |
| F-009 | 履歴選択再生 | 履歴から選択したMMLを再生 | 必須 | Phase 1 |
| F-010 | WAV出力 | 履歴からWAVファイルを生成 | 必須 | Phase 1 |
| F-011 | エラー表示 | MML文法エラーをわかりやすく表示 | 必須 | Phase 1 |
| F-012 | クリック音制御 | メトロノームのクリック音ON/OFF | 重要 | Phase 1 |
| F-013 | 再生アニメーション | 再生中にCLI上でビジュアルフィードバック | あれば良い | Phase 2 |
| F-014 | 波形シンセサイズ | エンベロープ、フィルター等の簡易シンセ機能 | あれば良い | Phase 2 |

### 3.2 ユーザーストーリー

#### US-001: MMLを即座に再生したい
- **ユーザー**: 作曲者
- **したいこと**: コマンドライン引数にMMLを入力すると即座に演奏が開始される
- **理由**: アイディアを思いついた瞬間に確認したい
- **受け入れ基準**:
  - [ ] `sine-mml play "CDEFGAB"` で即座に演奏が開始される
  - [ ] デフォルトでサイン波、音量中、BPM 120で再生される
  - [ ] Ctrl+Cで再生を中断できる
- **関連機能**: F-001, F-002, F-003, F-004, F-005

#### US-002: 過去のメロディを確認したい
- **ユーザー**: 作曲者
- **したいこと**: 以前入力したMMLの履歴を一覧で確認し、再生したい
- **理由**: 過去のアイディアを見返して発展させたい
- **受け入れ基準**:
  - [ ] `sine-mml history` で過去の履歴が日付順に表示される
  - [ ] 履歴にはID、作成日時、MML文字列が含まれる
  - [ ] `sine-mml play --history-id 5` で指定した履歴を再生できる
- **関連機能**: F-007, F-008, F-009

#### US-003: メロディをWAVファイルにしたい
- **ユーザー**: 作曲者
- **したいこと**: 気に入ったメロディをWAVファイルとして出力したい
- **理由**: 他のDAWで使用したり、共有したい
- **受け入れ基準**:
  - [ ] `sine-mml export --history-id 5 --output melody.wav` でWAV出力できる
  - [ ] WAV出力時はループしない（1回のみ再生）
  - [ ] 出力ファイルは標準的なオーディオプレイヤーで再生可能
- **関連機能**: F-010

#### US-004: MMLの記述ミスを知りたい
- **ユーザー**: 作曲者
- **したいこと**: MML記法を間違えた場合、どこが間違っているか知りたい
- **理由**: 正しいMMLを書けるようになりたい
- **受け入れ基準**:
  - [ ] 文法エラー時はエラー位置と内容を表示
  - [ ] エラーメッセージは日本語または英語でわかりやすく表示
  - [ ] 可能であれば修正のヒントを提示
- **関連機能**: F-011

#### US-005: 波形を切り替えたい
- **ユーザー**: 作曲者
- **したいこと**: サイン波、ノコギリ波、矩形波で音色を変えたい
- **理由**: メロディに合った音色を試したい
- **受け入れ基準**:
  - [ ] `--waveform sine|sawtooth|square` オプションで波形を指定できる
  - [ ] デフォルトはサイン波
  - [ ] 波形切り替えは即座に反映される
- **関連機能**: F-003

### 3.3 機能詳細

#### F-001: MMLパーサー

**概要**: MML文字列を解析し、音符・休符・オクターブ・音長などのデータ構造に変換

**対応するMML構文** (NuttX MML仕様準拠):
- **音符**: C, D, E, F, G, A, B (ド、レ、ミ、ファ、ソ、ラ、シ)
- **シャープ/フラット**: `#` または `+` (シャープ), `-` (フラット)
- **音長**: 1, 2, 4, 8, 16, 32, 64 (全音符=1, 四分音符=4)
- **付点**: `.` (音長を1.5倍)
- **休符**: `R` (音長指定可能)
- **オクターブ**: `O0`-`O8` (絶対指定), `<` (下げる), `>` (上げる)
- **デフォルト音長**: `L4` (以降の音符のデフォルト音長を設定)
- **テンポ**: `T120` (BPM指定)
- **音量**: `V0`-`V15` (音量レベル)

**入力**:
- MML文字列 (例: `"O4 L4 CDEFGAB > C"`)

**出力**:
- 音符データの配列（音程、音長、音量、オクターブなど）

**処理概要**:
1. MML文字列をトークンに分割
2. 各トークンを解析し、音符データ構造に変換
3. デフォルト値（音長、オクターブ、音量）を状態として管理
4. エラー位置と内容を記録

**ビジネスルール**:
- BR-001: 無効な音符（H, I, Jなど）はエラー
- BR-002: オクターブは0-8の範囲内
- BR-003: 音長は1,2,4,8,16,32,64のいずれか
- BR-004: 音量は0-15の範囲内

**制約事項**:
- 和音（コード）記法 `[CEG]` は Phase 1 では非対応
- タプレット記法 `{CEG}4` は Phase 1 では非対応

---

#### F-002: リアルタイム再生

**概要**: 解析したMMLデータを音声合成し、リアルタイムで再生

**入力**:
- 音符データ配列
- 波形タイプ（サイン波/ノコギリ波/矩形波）
- BPM
- 音量

**出力**:
- オーディオストリーム（標準出力デバイスに出力）

**処理概要**:
1. 音符データをサンプリング周波数44.1kHzのPCMデータに変換
2. fundspライブラリで指定波形を生成
3. cpalライブラリでオーディオデバイスに出力
4. Ctrl+C シグナルで中断可能

**ビジネスルール**:
- BR-005: サンプリングレート 44.1kHz 固定
- BR-006: モノラル出力のみ
- BR-007: 各音符間はレガート（音が途切れない）

**制約事項**:
- ポリフォニック（和音同時発音）は非対応

---

#### F-003: 波形選択

**概要**: サイン波、ノコギリ波、矩形波から音色を選択

**入力**:
- `--waveform` オプション: `sine` | `sawtooth` | `square`

**出力**:
- 選択された波形による音声出力

**処理概要**:
1. コマンドライン引数から波形タイプを取得
2. fundspの対応する波形生成関数を呼び出し
3. デフォルトはサイン波

**ビジネスルール**:
- BR-008: 無効な波形タイプはエラー

---

#### F-004: 音量調節

**概要**: 再生音量をコマンドラインオプションで調節

**入力**:
- `--volume` オプション: 0.0 - 1.0 (浮動小数点)

**出力**:
- 調整された音量での音声出力

**処理概要**:
1. コマンドライン引数から音量値を取得
2. PCMサンプルに音量係数を乗算
3. デフォルトは 0.5 (中程度)

**ビジネスルール**:
- BR-009: 音量は 0.0 - 1.0 の範囲内
- BR-010: 範囲外の値は最も近い有効値にクランプ

---

#### F-005: BPM調節

**概要**: 演奏テンポをコマンドラインオプションで調節

**入力**:
- `--bpm` オプション: 30 - 300 (整数)

**出力**:
- 指定BPMでの音声出力

**処理概要**:
1. コマンドライン引数からBPM値を取得
2. 音符の長さ（サンプル数）をBPMに応じて計算
3. デフォルトは 120 BPM

**ビジネスルール**:
- BR-011: BPMは 30 - 300 の範囲内
- BR-012: MML内の `T` コマンドがある場合はそちらを優先

---

#### F-006: ループ再生

**概要**: 演奏を繰り返し再生

**入力**:
- `--loop` フラグ（有無）

**出力**:
- ループ再生またはワンショット再生

**処理概要**:
1. `--loop` フラグがある場合、演奏終了後に先頭から再開
2. Ctrl+Cで中断されるまで継続

**ビジネスルール**:
- BR-013: デフォルトはワンショット再生（1回のみ）

---

#### F-007: 履歴保存

**概要**: 入力されたMML文字列を自動的にSQLiteデータベースに保存

**入力**:
- MML文字列
- 実行時のオプション（波形、音量、BPM）

**出力**:
- データベースへの保存（自動採番ID）

**処理概要**:
1. MML文字列が正常に解析された場合のみ保存
2. タイムスタンプ（UTC）を自動記録
3. SQLiteデータベースに INSERT

**データベーススキーマ**:
```sql
CREATE TABLE history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mml TEXT NOT NULL,
    waveform TEXT NOT NULL,
    volume REAL NOT NULL,
    bpm INTEGER NOT NULL,
    created_at TEXT NOT NULL  -- ISO 8601 format
);
```

**ビジネスルール**:
- BR-014: 文法エラーのあるMMLは保存しない
- BR-015: 同一MMLでも実行ごとに新規保存

---

#### F-008: 履歴一覧表示

**概要**: 保存された履歴を作成日時とともに一覧表示

**入力**:
- `history` サブコマンド

**出力**:
- 履歴の一覧（テーブル形式）

**処理概要**:
1. SQLiteから全履歴を作成日時降順で取得
2. 整形してCLIに表示

**出力形式例**:
```
ID | Created At          | MML                  | Waveform | BPM | Volume
---|---------------------|----------------------|----------|-----|-------
5  | 2026-01-10 10:30:00 | O4 L4 CDEFGAB        | sine     | 120 | 0.5
4  | 2026-01-09 14:20:00 | O5 T140 CRCRCR       | square   | 140 | 0.7
```

**ビジネスルール**:
- BR-016: 最新100件まで表示（ページネーション未実装）

---

#### F-009: 履歴選択再生

**概要**: 履歴から選択したMMLを再生

**入力**:
- `play --history-id <ID>` オプション

**出力**:
- 指定した履歴の再生

**処理概要**:
1. 指定IDの履歴をSQLiteから取得
2. 保存されたオプション（波形、BPM、音量）で再生
3. IDが存在しない場合はエラー

**ビジネスルール**:
- BR-017: 存在しないIDはエラーメッセージを表示

---

#### F-010: WAV出力

**概要**: 履歴から選択したMMLをWAVファイルとして出力

**入力**:
- `export --history-id <ID> --output <filename>`

**出力**:
- WAVファイル（44.1kHz, 16bit, モノラル）

**処理概要**:
1. 指定IDの履歴を取得
2. MMLを音声データに変換（ループなし）
3. WAVフォーマットでファイル出力

**ビジネスルール**:
- BR-018: WAV出力時はループしない（1回のみ）
- BR-019: 既存ファイルは確認なく上書き

---

#### F-011: エラー表示

**概要**: MML文法エラーをわかりやすく表示

**入力**:
- パースエラー情報（位置、内容）

**出力**:
- エラーメッセージ（標準エラー出力）

**処理概要**:
1. エラー位置（文字位置）を特定
2. エラー内容を人間可読な形式で表示

**出力形式例**:
```
Error: Invalid note 'H' at position 12
  O4 L4 CDEFGAH
            ^
Expected: A-G, R, or octave/tempo command
```

**ビジネスルール**:
- BR-020: エラー時は終了コード 1 で終了

---

#### F-012: クリック音制御

**概要**: メトロノームのクリック音をON/OFF

**入力**:
- `--metronome` フラグ（有無）

**出力**:
- クリック音の有無

**処理概要**:
1. `--metronome` フラグがある場合、各拍の先頭で短いクリック音を再生
2. クリック音は高周波短音（1kHz, 10ms）

**ビジネスルール**:
- BR-021: デフォルトはクリック音OFF

---

#### F-013: 再生アニメーション

**概要**: 再生中にCLI上でビジュアルフィードバック

**入力**:
- 再生中の音符データ

**出力**:
- CLIアニメーション（スピナー、プログレスバーなど）

**処理概要**:
1. 現在再生中の音符を表示
2. プログレスバーで進捗を表示

**出力形式例**:
```
Playing: C4 D4 E4 F4 ...
[████████----------] 40% (BPM: 120)
```

**ビジネスルール**:
- BR-022: アニメーションは標準エラー出力に表示（リダイレクト可能）

---

## 4. 非機能要件

### 4.1 性能要件

| ID | 要件 | 目標値 | 測定方法 |
|----|------|--------|----------|
| NFR-P-001 | MMLパースレイテンシ | 100ms以内（1000文字のMML） | ユニットテスト |
| NFR-P-002 | 音声出力レイテンシ | 50ms以内 | オーディオバッファ測定 |
| NFR-P-003 | 履歴取得速度 | 10ms以内（1000件） | SQLiteクエリ測定 |
| NFR-P-004 | WAV出力速度 | 1分のオーディオを5秒以内 | ベンチマーク |

### 4.2 可用性要件

| ID | 要件 | 目標値 |
|----|------|--------|
| NFR-A-001 | クラッシュ率 | 0%（正常なMML入力） |
| NFR-A-002 | エラーハンドリング | すべてのエラーを捕捉し適切に処理 |

### 4.3 セキュリティ要件

| ID | 要件 | 詳細 |
|----|------|------|
| NFR-S-001 | 入力サニタイゼーション | MML文字列の長さ制限（10,000文字） |
| NFR-S-002 | SQLインジェクション対策 | プリペアドステートメント使用 |
| NFR-S-003 | ファイル出力制限 | WAV出力時のパストラバーサル対策 |

### 4.4 ユーザビリティ要件

| ID | 要件 | 詳細 |
|----|------|------|
| NFR-U-001 | 対応OS | macOS, Linux, Windows (クロスプラットフォーム) |
| NFR-U-002 | CLIヘルプ | `--help` で全オプションの説明を表示 |
| NFR-U-003 | エラーメッセージ | 日本語または英語で明確に表示 |
| NFR-U-004 | インストール方法 | `cargo install` または バイナリ配布 |

### 4.5 保守性要件

| ID | 要件 | 詳細 |
|----|------|------|
| NFR-M-001 | コードカバレッジ | 80%以上 |
| NFR-M-002 | ドキュメント | README, APIドキュメント（rustdoc） |
| NFR-M-003 | ログ | デバッグ時に `RUST_LOG` 環境変数で詳細ログ出力 |
| NFR-M-004 | テスト配置 | `tests/` ディレクトリに統合テスト配置 |

---

## 5. 制約条件

### 5.1 技術的制約

| 制約 | 詳細 | 理由 |
|------|------|------|
| 言語 | Rust | 高性能・安全性・クロスプラットフォーム |
| オーディオライブラリ | fundsp + cpal | 波形生成とI/Oの分離、有名ライブラリ |
| データベース | SQLite (rusqlite) | 軽量、組み込み可能 |
| CLIフレームワーク | clap | Rust標準的、豊富な機能 |
| テスト配置 | `tests/` ディレクトリ | トークン消費量削減 |

### 5.2 ビジネス制約

| 制約 | 詳細 |
|------|------|
| 予算 | オープンソース（無償） |
| スケジュール | Phase 1: 4週間、Phase 2: 2週間 |
| リソース | 個人開発（1名） |

### 5.3 法規制・コンプライアンス

| 要件 | 詳細 |
|------|------|
| ライセンス | MIT License（依存ライブラリと互換性あり） |
| 著作権 | 生成されたWAVファイルの著作権はユーザーに帰属 |

---

## 6. 外部インターフェース

### 6.1 外部システム連携

なし（スタンドアロンアプリケーション）

### 6.2 データ移行

| 移行元 | データ種別 | 件数目安 | 移行方式 |
|--------|-----------|---------|---------|
| なし（新規プロジェクト） | - | - | - |

---

## 7. 前提条件と依存関係

### 7.1 前提条件

- Rustコンパイラ（1.70以上）がインストールされていること
- オーディオ出力デバイスが利用可能であること
- SQLite 3.x が利用可能であること（rusqliteが静的リンク）

### 7.2 依存関係

| 依存先 | 内容 | 影響 |
|--------|------|------|
| fundsp | 波形生成 | 変更時は音質に影響 |
| cpal | オーディオI/O | 変更時は再生に影響 |
| rusqlite | SQLite操作 | 変更時は履歴機能に影響 |
| clap | CLI引数解析 | 変更時はCLI仕様に影響 |

---

## 8. リスクと課題

### 8.1 リスク一覧

| ID | リスク | 影響度 | 発生確率 | 対策 |
|----|--------|--------|---------|------|
| R-001 | fundsp/cpalのAPI変更 | 中 | 低 | バージョン固定、定期的な依存更新 |
| R-002 | オーディオレイテンシ問題 | 高 | 中 | バッファサイズ調整、性能テスト |
| R-003 | クロスプラットフォーム互換性 | 中 | 中 | CI/CDで複数OS テスト |
| R-004 | MMLパーサーのバグ | 高 | 中 | 豊富なテストケース、エッジケース対応 |

### 8.2 未解決課題

| ID | 課題 | 担当 | 期限 |
|----|------|------|------|
| I-001 | ~~fundsp/cpalの詳細な使用方法確認~~ **解決済** | 開発チーム | ~~2026-01-17~~ |
| I-002 | WAVファイル出力ライブラリの選定 (hound推奨) | 開発チーム | 2026-01-17 |
| I-003 | CLIアニメーションライブラリの選定 (indicatif推奨) | 開発チーム | 2026-01-24 |
| I-004 | カバレッジ測定ツールの選定 (tarpaulin等) | 開発チーム | 2026-01-17 |

---

## 9. 技術スタック（詳細）

### 9.1 コア技術

| レイヤー | 技術 | バージョン | 役割 |
|---------|------|----------|------|
| 言語 | Rust | 1.70+ | アプリケーション全体 |
| オーディオ生成 | fundsp | 0.18+ | サイン波/ノコギリ波/矩形波生成（DSPグラフ） |
| オーディオI/O | cpal | 0.15+ | クロスプラットフォーム音声出力（低レイテンシ） |
| データベース | rusqlite | 0.31+ | SQLite操作 |
| CLI | clap | 4.x | コマンドライン引数解析 |
| WAV出力 | hound (候補) | 3.x | WAVファイルエンコード |
| CLIアニメーション | indicatif (候補) | 0.17+ | プログレスバー・スピナー |

### 9.2 技術選定理由（詳細）

#### fundsp（音声合成）
- **選定理由**:
  - 豊富な波形生成機能（`sine_hz`, `saw_hz`, `square_hz`, `triangle_hz`）
  - DSPグラフによる柔軟な音声処理（`>>`, `|`, `+`演算子）
  - SIMD最適化による高性能（64サンプルブロック処理、f32x8/f64x4）
  - リアルタイム対応（ゼロアロケーション、`no_std`サポート）
  - GitHub Stars: 2.1k、活発なメンテナンス
- **使用例**:
  ```rust
  use fundsp::prelude::*;
  let sine = sine_hz(440.0);  // 440Hz サイン波
  let saw = saw_hz(220.0);    // 220Hz ノコギリ波
  let square = square_hz(110.0); // 110Hz 矩形波
  ```

#### cpal（オーディオI/O）
- **選定理由**:
  - クロスプラットフォーム（Windows/macOS/Linux/WASM/Android）
  - 超低レイテンシ（ハードウェア依存、バッファサイズ調整可能）
  - コールバックベースの効率的な処理
  - fundspとの統合パターンが確立
  - GitHub Stars: 3.5k、業界標準
- **rodio非採用理由**: 高レベルすぎる、レイテンシが高い（~20ms）、リアルタイム合成には不向き
- **統合パターン**:
  ```rust
  let stream = device.build_output_stream(&config, move |data: &mut [f32], _| {
      for frame in data.chunks_mut(2) {
          let (l, r) = audio_graph.get_stereo();
          frame[0] = l as f32;
          frame[1] = r as f32;
      }
  }, err_fn, None)?;
  ```

### 9.2 開発ツール

| ツール | 用途 |
|--------|------|
| cargo | ビルド・依存管理 |
| rustfmt | コードフォーマット |
| clippy | Lint |
| cargo-tarpaulin (候補) | カバレッジ測定 |
| GitHub Actions | CI/CD |

### 9.3 アーキテクチャ設計パターン

#### モジュール構成（推奨）
```
src/
├── main.rs              # CLIエントリポイント
├── mml/
│   ├── mod.rs          # MMLパーサーモジュール
│   ├── parser.rs       # パーサー実装（再帰下降）
│   ├── ast.rs          # 抽象構文木（Note, Rest, Octave等）
│   └── error.rs        # パースエラー型
├── audio/
│   ├── mod.rs          # オーディオモジュール
│   ├── synthesizer.rs  # fundspによる波形生成
│   ├── player.rs       # cpalによる再生制御
│   └── waveform.rs     # 波形タイプ列挙型
├── db/
│   ├── mod.rs          # データベースモジュール
│   ├── schema.rs       # テーブル定義・マイグレーション
│   └── history.rs      # 履歴CRUD操作
└── cli/
    ├── mod.rs          # CLIモジュール
    └── commands.rs     # サブコマンド実装（play, history, export）
```

#### fundsp + cpal統合パターン
```rust
// 1. fundspでDSPグラフ作成
let mut synth: Box<dyn AudioUnit64> = Box::new(
    sine_hz(440.0) * 0.5  // 440Hz サイン波、音量50%
);
synth.set_sample_rate(44100.0);

// 2. cpalでオーディオストリーム構築
let stream = device.build_output_stream(
    &config,
    move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
        for frame in output.chunks_mut(channels) {
            let (left, right) = synth.get_stereo();
            frame[0] = left as f32;
            if channels == 2 { frame[1] = right as f32; }
        }
    },
    |err| eprintln!("Stream error: {}", err),
    None,
)?;
```

### 9.4 CI/CDパイプライン

| ステージ | 内容 |
|---------|------|
| Lint | `cargo clippy -- -D warnings` |
| Format | `cargo fmt -- --check` |
| Type Check | `cargo check` |
| Test | `cargo test` (tests/ 配下の統合テスト) |
| Coverage | `cargo tarpaulin --out Xml` (要選定 I-004) |

---

## 10. 用語集

| 用語 | 定義 |
|------|------|
| MML | Music Macro Language。文字列で音楽を記述する言語 |
| サイン波 | 正弦波。基本的な波形で柔らかい音色 |
| ノコギリ波 | のこぎり状の波形。明るい音色 |
| 矩形波 | 矩形状の波形。レトロゲーム的な音色 |
| BPM | Beats Per Minute。1分間あたりの拍数 |
| モノフォニック | 単音のみ。和音同時発音不可 |
| ポリフォニック | 和音同時発音可能 |
| レイテンシ | 入力から出力までの遅延時間 |
| PCM | Pulse Code Modulation。デジタル音声の標準形式 |
| fundsp | Rust製DSPライブラリ。波形生成に特化 |
| cpal | Cross-Platform Audio Library。Rust製オーディオI/O |

---

## 11. 参考リンク

### 11.1 MML仕様
- [NuttX MML Parser Documentation](https://nuttx.apache.org/docs/latest/applications/audioutils/mml_parser/index.html)

### 11.2 Rustオーディオライブラリ
- **fundsp**:
  - [GitHub Repository](https://github.com/SamiPerttu/fundsp) (2.1k stars)
  - [docs.rs Documentation](https://docs.rs/fundsp/)
  - Community Example: [fundsp-example](https://github.com/mochreach/fundsp-example)
- **cpal**:
  - [GitHub Repository](https://github.com/RustAudio/cpal) (3.5k stars)
  - [docs.rs Documentation](https://docs.rs/cpal/)
  - [Examples: synth_tones.rs](https://github.com/RustAudio/cpal/blob/master/examples/synth_tones.rs)
- **rodio** (参考):
  - [GitHub Repository](https://github.com/RustAudio/rodio) (2.5k stars)
  - 注: 本プロジェクトではレイテンシ要件のため非採用
- **hound** (WAV I/O候補):
  - [GitHub Repository](https://github.com/ruuda/hound)
  - [docs.rs Documentation](https://docs.rs/hound/)

### 11.3 Rust SQLite
- [rusqlite Documentation](https://docs.rs/rusqlite/)
- [Rust Cookbook - SQLite](https://rust-lang-nursery.github.io/rust-cookbook/database/sqlite.html)

### 11.4 Rust CLI
- [clap Documentation](https://docs.rs/clap/)
- [indicatif Documentation](https://docs.rs/indicatif/) (プログレスバー候補)

---

## 12. 実装ガイダンス（技術調査結果より）

### 12.1 fundsp 波形生成パターン

#### 基本的な波形生成
```rust
use fundsp::prelude::*;

// 固定周波数の波形
let sine = sine_hz(440.0);        // A4 (440Hz) サイン波
let saw = saw_hz(220.0);          // A3 ノコギリ波
let square = square_hz(110.0);    // A2 矩形波
let triangle = triangle_hz(880.0); // A5 三角波

// 波形の合成（加算合成）
let additive = (sine_hz(440.0) + saw_hz(440.0) * 0.5) 
    >> lowpass_hz(1000.0, 1.0);  // ローパスフィルター適用
```

#### MMLに対応する動的周波数制御
MMLでは音符ごとに周波数が変わるため、fundspのDSPグラフを動的に再構築する必要があります。

**推奨アプローチ**:
1. **音符ごとにグラフ再構築** (シンプル、Phase 1推奨)
2. **周波数変調** (高度、Phase 2で検討)

```rust
// アプローチ1: 音符ごとに再構築
for note in mml_ast {
    let freq = note_to_frequency(note);
    let mut synth = Box::new(sine_hz(freq) * note.volume);
    synth.set_sample_rate(44100.0);
    // cpalストリームに渡して note.duration 秒間再生
}

// アプローチ2: 周波数変調 (Phase 2)
// 外部からシグナルを入力してリアルタイムに周波数を変更
let freq_signal = var(\u0026freq_control);  // 可変周波数
let synth = freq_signal >> sine();
```

### 12.2 cpal オーディオ出力パターン

#### 基本的なストリーム構築
```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

let host = cpal::default_host();
let device = host.default_output_device().expect("No output device");
let config = device.default_output_config().expect("No output config");

let sample_rate = config.sample_rate().0 as f64;
let channels = config.channels() as usize;

let stream = device.build_output_stream(
    &config.into(),
    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        // fundspのget_stereo()を呼び出してサンプルを取得
        for frame in data.chunks_mut(channels) {
            let (left, right) = audio_graph.get_stereo();
            frame[0] = left as f32;
            if channels == 2 { frame[1] = right as f32; }
        }
    },
    |err| eprintln!("Stream error: {}", err),
    None,
)?;

stream.play()?;
```

#### レイテンシ最適化
```rust
use cpal::{BufferSize, StreamConfig};

let mut config: StreamConfig = device.default_output_config()?.into();
config.buffer_size = BufferSize::Fixed(256);  // 低レイテンシ設定（256サンプル）

// 256サンプル @ 44.1kHz = 約5.8msのレイテンシ
```

### 12.3 MMLパーサー実装パターン

#### 手動再帰下降パーサー（推奨）
MMLは状態依存（オクターブ、デフォルト音長など）のため、パーサーコンビネータより手動実装が適切です。

```rust
pub struct MmlParser {
    input: Vec<char>,
    pos: usize,
    // 状態
    current_octave: u8,       // デフォルト 4
    default_length: u8,       // デフォルト 4 (四分音符)
    current_volume: u8,       // デフォルト 8 (0-15)
    current_bpm: u16,         // デフォルト 120
}

impl MmlParser {
    pub fn parse(&mut self) -> Result<Vec<Event>, ParseError> {
        let mut events = Vec::new();
        while !self.is_eof() {
            match self.current_char() {
                'A'..='G' => events.push(self.parse_note()?),
                'R' => events.push(self.parse_rest()?),
                'O' => self.parse_octave()?,
                'L' => self.parse_default_length()?,
                'T' => self.parse_tempo()?,
                'V' => self.parse_volume()?,
                '<' => self.current_octave = self.current_octave.saturating_sub(1),
                '>' => self.current_octave = (self.current_octave + 1).min(8),
                ' ' | '\n' | '\t' => self.advance(),
                c => return Err(ParseError::UnexpectedChar(c, self.pos)),
            }
        }
        Ok(events)
    }
    
    fn parse_note(&mut self) -> Result<Event, ParseError> {
        let note_char = self.current_char();
        self.advance();
        
        // シャープ/フラット
        let accidental = match self.current_char() {
            '#' | '+' => { self.advance(); Accidental::Sharp },
            '-' => { self.advance(); Accidental::Flat },
            _ => Accidental::Natural,
        };
        
        // 音長
        let length = if self.current_char().is_ascii_digit() {
            self.parse_length()?
        } else {
            self.default_length
        };
        
        // 付点
        let dotted = self.parse_dots();
        
        Ok(Event::Note {
            pitch: note_to_pitch(note_char, accidental, self.current_octave),
            length,
            dotted,
            volume: self.current_volume,
        })
    }
}
```

### 12.4 SQLite スキーマとトランザクション

#### テーブル作成
```rust
use rusqlite::{Connection, Result};

pub fn init_db(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            mml TEXT NOT NULL,
            waveform TEXT NOT NULL,
            volume REAL NOT NULL,
            bpm INTEGER NOT NULL,
            created_at TEXT NOT NULL
        )",
        [],
    )?;
    
    // インデックス作成（作成日時での検索を高速化）
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_created_at ON history(created_at DESC)",
        [],
    )?;
    
    Ok(conn)
}
```

#### トランザクション使用
```rust
pub fn save_history(
    conn: &Connection,
    mml: &str,
    waveform: &str,
    volume: f32,
    bpm: u16,
) -> Result<i64> {
    let tx = conn.transaction()?;
    
    tx.execute(
        "INSERT INTO history (mml, waveform, volume, bpm, created_at)
         VALUES (?1, ?2, ?3, ?4, datetime('now'))",
        (mml, waveform, volume, bpm),
    )?;
    
    let id = tx.last_insert_rowid();
    tx.commit()?;
    
    Ok(id)
}
```

### 12.5 clap CLI定義

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sine-mml")]
#[command(about = "MML Synthesizer CLI", version, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Play MML directly or from history
    Play {
        /// MML string to play
        mml: Option<String>,
        
        /// Play from history by ID
        #[arg(long)]
        history_id: Option<i64>,
        
        /// Waveform type
        #[arg(short, long, default_value = "sine")]
        waveform: Waveform,
        
        /// Volume (0.0 - 1.0)
        #[arg(short, long, default_value = "0.5")]
        volume: f32,
        
        /// BPM (30 - 300)
        #[arg(short, long, default_value = "120")]
        bpm: u16,
        
        /// Loop playback
        #[arg(long)]
        r#loop: bool,
        
        /// Enable metronome
        #[arg(long)]
        metronome: bool,
    },
    
    /// Show history
    History,
    
    /// Export history to WAV file
    Export {
        /// History ID to export
        #[arg(long)]
        history_id: i64,
        
        /// Output file path
        #[arg(short, long)]
        output: String,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum Waveform {
    Sine,
    Sawtooth,
    Square,
}
```

---

## 13. 変更履歴

| バージョン | 日付 | 変更内容 | 作成者 |
|-----------|------|----------|--------|
| 1.0.0 | 2026-01-10 | 初版作成 | Antigravity |
| 1.1.0 | 2026-01-10 | fundsp/cpal技術調査結果を反映、実装ガイダンス追加 | Antigravity |
