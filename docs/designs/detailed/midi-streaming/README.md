# MIDIストリーミング 詳細設計

## 概要

本フォルダには、F-031（MIDIストリーミング）機能の詳細設計書を格納します。

## 関連文書

- **要件定義書**: [REQ-CLI-006_MIDI-Streaming-Tuplet.md](../../../requirements/REQ-CLI-006_MIDI-Streaming-Tuplet.md)
- **基本設計書**: [BASIC-CLI-006_MIDI-Streaming-Tuplet.md](../../basic/BASIC-CLI-006_MIDI-Streaming-Tuplet.md)

## 詳細設計書一覧

| ドキュメントID | 名称 | ステータス |
|----------------|------|------------|
| DET-MIDI-001 | MIDIデバイス管理詳細設計 | 未着手 |
| DET-MIDI-002 | MIDIメッセージ送信詳細設計 | 未着手 |
| DET-MIDI-003 | MIDIエラーハンドリング詳細設計 | 未着手 |

## 機能概要

- `--midi-out` オプションでMIDIデバイスにリアルタイム送信
- `--midi-channel` でMIDIチャンネル指定（1-16）
- `--midi-list` で利用可能なMIDIデバイス一覧を表示
- 絶対時刻管理によるドリフトフリーな送信タイミング

## 技術スタック

- **MIDIライブラリ**: midir 0.9

## エラーコード

| コード | 説明 |
|--------|------|
| MML-E015 | MIDIデバイスが見つからない |
| MML-E016 | MIDIデバイス接続エラー |
| MML-E017 | MIDIメッセージ送信エラー |
| MML-E018 | 無効なMIDIデバイスID |
| MML-E019 | MIDIデバイス切断 |
| MML-E024 | 無効なMIDIチャンネル |
