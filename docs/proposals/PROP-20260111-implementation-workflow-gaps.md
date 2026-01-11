---
id: PROP-20260111-001
title: 実装ワークフローの「統合漏れ」および「スタブ残存」防止策の導入
status: Draft
author: Sisyphus
date: 2026-01-11
---

# 実装ワークフローの「統合漏れ」および「スタブ残存」防止策の導入

## 1. Executive Summary
sine-mml開発において発生した「機能実装済みだがCLIから利用不能」「引数定義のみで中身が空」という問題を解決するため、**CLI-Backend対応マトリクスの導入**と**E2E統合テストの義務化**を提案します。これにより、実効性のない「見かけ上のIssue完了」を排除し、開発の透明性と品質を向上させます。

## 2. Problem & Context (As-Is)
### 現状の課題
1. **CLI統合の忘却**: 個別の機能（play, history等）が完成しても、`main.rs` への登録が漏れており、バイナリとして動作しない状態でIssueがクローズされた（Issue #13, #14）。
2. **ゾンビ・オプション（スタブ）の残存**: `--bpm` や `--metronome` のように、CLI引数としては定義されているが、バックエンドのロジックに反映されていない、あるいは実装が空の機能が「完了」扱いとなっている。
3. **Epic進捗の不透明性**: Epic Issue #1 の機能リスト(F-001〜F-013)と実際のコードベース・Sub-issueの進捗が同期しておらず、正確な完成度が把握できない。

### 発生頻度・影響範囲
- **頻度**: 新機能追加時およびCLIインターフェース変更時に毎回発生するリスクがある。
- **影響**: 「動かないコード」がマージされることで、後続のタスクでの手戻りが発生。手戻りコストは、初期実装時の2〜3倍に膨らむ傾向がある。

## 3. Solution (To-Be)
### 改善案
1. **CLI-Backend対応マトリクス (Capability Matrix) の作成**:
   - `docs/capabilities.md` を作成し、CLI引数とバックエンド実装の紐付けを管理。実装が空のものは `(Stub)` と明記し、完了時のみチェックを入れる。
2. **PRテンプレートへの「統合テスト項目」追加**:
   - 単体テスト(Unit Test)だけでなく、`cargo run -- play ...` 等の実際のCLIコマンドを用いた統合テストの結果をPRに添付することを義務化。
3. **E2E（エンドツーエンド）テストの導入**:
   - `tests/cli_integration.rs` を作成。`assert_cmd` クレート等を用い、各オプション（`--bpm` 等）がバイナリレベルで正しく動作し、期待される副作用（再生時間の変化、DB保存等）を生むかを自動検証する。
4. **Epic進捗の自動同期**:
   - GitHub Actionsを用いて、Sub-issueのステータス変更時にEpicのチェックリストを自動更新するワークフローを検討。

### 期待される効果 (ROI)
| 項目 | 現状 (Cost) | 改善後 (Benefit) | 効果 (Delta) |
|------|-------------|------------------|--------------|
| 手戻り修正工数 | 4-8時間/Epic | 1時間以内 | **75-85% 削減** |
| 品質保証(QA)時間 | 手動確認に30分/PR | 自動テストで5分 | **83% 削減** |
| 進捗管理コスト | 毎週30分の同期会議 | リアルタイム反映 | **週30分削減** |
| ユーザー信頼性 | 「動かない」報告のリスク高 | 常に動作保証 | リスクの大幅低減 |

## 4. Implementation Details
### 具体的な変更点
1. **`src/audio/synthesizer.rs` の拡張**:
   - `Synthesizer::synthesize` メソッドのシグネチャを変更：
     ```rust
     pub fn synthesize(&mut self, mml: &Mml, initial_bpm: u16, metronome: bool) -> Result<Vec<f32>, Box<dyn Error>>
     ```
   - `initial_bpm` を内部の `bpm` 状態の初期値として使用。
   - `metronome` が true の場合、各拍の先頭サンプルに `generate_click_samples` の出力をミックスするロジックを追加。

2. **`src/cli/handlers.rs` の修正**:
   - `play_handler` 内で、`args.bpm` および `args.metronome` を `synth.synthesize()` に確実に渡すよう修正。

3. **E2Eテスト基盤の構築**:
   - `tests/cli_integration.rs` を作成：
     ```rust
     #[test]
     fn test_play_command_with_bpm() {
         let mut cmd = Command::cargo_bin("sine-mml").unwrap();
         cmd.arg("play").arg("C").arg("--bpm").arg("300");
         // 実行時間が短くなることを検証
     }
     ```

4. **ドキュメント整備**:
   - `docs/capabilities.md` を作成し、機能実装状況（Implemented / Stub / Planned）を一覧化。

### 依存関係・リスク
- **依存**: `assert_cmd`, `predicates` クレートを `[dev-dependencies]` に追加。
- **リスク**: 合成ロジックへのメトロノームミックス追加による、クリッピング（音割れ）のリスク。ゲイン調整（リミッター）の検討が必要。

## 5. Next Actions
- [ ] GitHub Issue: 「E2Eテスト基盤の構築とCLI統合テストの追加」を作成
- [ ] GitHub Issue: 「BPM設定およびメトロノーム機能の完全実装」を作成
- [ ] `docs/capabilities.md` の初期版を作成
