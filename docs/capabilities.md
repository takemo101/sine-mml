# sine-mml CLI-Backend Capability Matrix

**Project**: sine-mml  
**Version**: v2.0 (current)  
**Last Updated**: 2026-01-12

---

## Purpose

This document tracks the implementation status of all CLI options and their corresponding backend implementations in the sine-mml project. It serves to:

1. **Prevent stub features** from shipping to production
2. **Guide developers** on what is and isn't implemented
3. **Track implementation progress** for new features
4. **Ensure CLI-Backend parity** during development

---

## Current Implementation Status (v2.0)

| CLI Option | Backend Implementation | Status | Notes |
|------------|----------------------|--------|-------|
| `play <MML>` | `mml::parse()` + `Synthesizer::synthesize()` | ✅ Implemented | Core functionality - MML parsing and synthesis |
| `--waveform <TYPE>` | `WaveformType` enum (Sine, Sawtooth, Square) | ✅ Implemented | 3 waveform types supported |
| `--volume <0.0-1.0>` | `Synthesizer.volume` + `validate_volume()` | ✅ Implemented | Range validated by clap |
| `--loop-play` | `AudioPlayer.play(loop=true)` | ✅ Implemented | Infinite loop playback, history saved before loop |
| `--metronome` | `Synthesizer::mix_metronome()` + `generate_noise_click()` | ✅ Implemented | Noise-based click sound with exponential envelope |
| `--metronome-beat <4\|8\|16>` | `Synthesizer::beat_interval_seconds()` | ✅ Implemented | Beat selection (4/8/16), default=4 |
| `--metronome-volume <0.0-1.0>` | `Synthesizer::generate_noise_click(volume)` | ✅ Implemented | Metronome volume control, default=0.3 |
| `--history-id <ID>` | `Database.get_by_id()` | ✅ Implemented | Replay from history database |
| `history` | `Database.list(limit=20)` + comfy-table display | ✅ Implemented | Lists last 20 entries with formatted table |
| `export --history-id <ID>` | `Database.get_by_id()` + `exporter::export_wav()` | ✅ Implemented | WAV file export from history |
| `export --output <PATH>` | `std::path::Path` + path traversal validation | ✅ Implemented | Security: blocks `..` in path |

---

## Removed Features (v2.0)

| CLI Option | Removal Reason | Migration |
|------------|---------------|-----------|
| `--bpm <30-300>` | Redundant with MML `T` command | Use `T120` in MML string instead of `--bpm 120` |

---

## Feature Implementation Details

### ✅ Fully Implemented Features

#### `play <MML>`
- **Backend**: `mml::parse()` parses MML string into AST
- **Backend**: `Synthesizer::synthesize()` generates audio samples
- **Test Coverage**: Unit tests in `mml/parser.rs`, `audio/synthesizer.rs`
- **Location**: `src/cli/handlers.rs::play_handler()`

#### `--waveform`
- **Backend**: `WaveformType` enum (Sine, Sawtooth, Square)
- **Conversion**: CLI arg → audio module enum
- **Test Coverage**: Integration tests verify all 3 waveforms
- **Location**: `src/cli/args.rs::Waveform`, `src/audio/waveform.rs`

#### `--volume`
- **Backend**: `Synthesizer.volume` field (0.0-1.0)
- **Validation**: `validate_volume()` enforces range
- **Conversion**: f32 (0.0-1.0) → u8 (0-100) internally
- **Location**: `src/cli/handlers.rs::play_handler()` (line 48-49)

#### `--loop-play`
- **Backend**: `AudioPlayer.play(loop=true)` enables infinite loop
- **User Exit**: Ctrl+C to terminate
- **History Saving**: Fixed in v1.0 - history saved before loop starts
- **Location**: `src/audio/player.rs`, `src/cli/handlers.rs` (line 65)

#### `history`
- **Backend**: `Database.list(limit=20)` queries SQLite
- **Display**: `comfy-table` for formatted output
- **Columns**: ID, MML (truncated), Waveform, Volume, BPM, Created At
- **Location**: `src/cli/handlers.rs::history_handler()` (line 105-136)

#### `export`
- **Backend**: `exporter::export_wav()` writes 16-bit PCM WAV
- **Security**: Path traversal validation (blocks `..`)
- **Format**: 44100 Hz, 16-bit, mono
- **Location**: `src/cli/handlers.rs::export_handler()` (line 146-184)

---

### v2.0 New Features

#### `--metronome` (Enhanced)
- **Backend**: `Synthesizer::mix_metronome()` + `generate_noise_click()`
- **Sound**: Noise-based click with exponential envelope using fundsp
- **Integration**: Mixed with music samples before playback
- **Location**: `src/audio/synthesizer.rs`

#### `--metronome-beat`
- **Purpose**: Select metronome beat pattern
- **Values**: 4 (quarter note), 8 (eighth note), 16 (sixteenth note)
- **Backend Module**: `Synthesizer::beat_interval_seconds()`
- **Validation**: Clap custom validator (4, 8, 16 only)
- **Default**: 4
- **Location**: `src/cli/args.rs`

#### `--metronome-volume`
- **Purpose**: Independent volume control for metronome
- **Range**: 0.0-1.0
- **Backend Module**: `Synthesizer::generate_noise_click(volume)`
- **Default**: 0.3 (subtle, non-intrusive)
- **Location**: `src/cli/args.rs`

#### Loop History Saving
- **Change**: History saved BEFORE playback loop starts
- **Benefit**: Loop playback now correctly saves to history
- **Location**: `src/cli/handlers.rs::play_handler()`

#### Audio Normalization
- **Purpose**: Prevent digital clipping when metronome + music combined
- **Backend**: `Synthesizer::normalize_samples()`
- **Location**: `src/audio/synthesizer.rs`

---

### ⛔ Removed Features (v2.0)

#### `--bpm` (Removed)
- **Reason**: Redundant with MML `T` command (e.g., `T120`)
- **Replacement**: Use MML `T` command inline (e.g., `"T140 CDEFGAB"`)
- **Error**: Using `--bpm` now returns "unexpected argument" error

---

## Legend

| Symbol | Status | Description |
|--------|--------|-------------|
| ✅ | **Implemented** | Fully implemented and tested - CLI option works as expected |
| 🚧 | **In Progress** | Partially implemented - may be a stub or incomplete functionality |
| ❌ | **Not Implemented** | Planned feature but not yet started |
| ⛔ | **Removed** | Feature has been removed from the CLI |

---

## How to Update This Document

### When to Update

| Event | Action | Example |
|-------|--------|---------|
| **New CLI option added** | Add new row with Status ❌ | Adding `--reverb` flag |
| **Implementation started** | Change Status ❌ → 🚧 | Starting work on metronome sound generation |
| **Implementation completed** | Change Status 🚧 → ✅ | Metronome fully working with tests |
| **Feature deprecated** | Change Status ✅ → ⚠️ + add removal note | Deprecating `--bpm` |
| **Feature removed** | Delete row + note in changelog | Removing `--bpm` in v2.0 |

### Update Process

1. **Edit this file** (`docs/capabilities.md`)
2. **Update the table** with the current status
3. **Update "Last Updated"** date at the top
4. **Include in your PR** - add to PR description
5. **Check PR template** - mark capability matrix checkbox

### PR Template Checklist

When creating a PR that adds or modifies CLI options:

```markdown
## Capability Matrix Update

- [ ] `docs/capabilities.md` updated (if CLI options changed)
- [ ] Status accurately reflects implementation state (✅/🚧/❌)
- [ ] Backend implementation module documented
- [ ] Notes column includes version info (if applicable)
```

---

## Version History

| Version | Date | Changes | Author |
|---------|------|---------|--------|
| 2.0.0 | 2026-01-12 | Updated for v2.0 release: metronome, normalization, --bpm removed | - |
| 1.0.0 | 2026-01-11 | Initial capability matrix created (F-020) | - |

---

## References

- **Basic Design**: `docs/designs/basic/BASIC-CLI-002_MML-Synthesizer-Enhancement.md`
- **Requirements**: `docs/requirements/REQ-CLI-002_MML-Synthesizer-Enhancement.md`
- **CLI Arguments**: `src/cli/args.rs`
- **CLI Handlers**: `src/cli/handlers.rs`

---

**Note**: This document is the **single source of truth** for CLI-Backend implementation status. Keep it updated with every PR that touches CLI functionality.
