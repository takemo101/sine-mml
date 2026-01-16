use crate::audio::waveform::{create_node, midi_to_frequency, WaveformType};
use crate::mml::{Command, Mml, Note, VolumeValue};
use fundsp::hacker::{highpass_hz, noise};
use std::error::Error;

pub struct Synthesizer {
    pub sample_rate: u32,
    pub volume: u8,
    pub waveform_type: WaveformType,
}

impl Synthesizer {
    #[must_use]
    pub fn new(sample_rate: u32, volume: u8, waveform_type: WaveformType) -> Self {
        Self {
            sample_rate,
            volume,
            waveform_type,
        }
    }

    /// Synthesize MML into audio samples.
    ///
    /// # Errors
    /// Returns an error if synthesis fails (though currently it mostly succeeds).
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn synthesize(&mut self, mml: &Mml) -> Result<Vec<f32>, Box<dyn Error>> {
        let mut samples = Vec::new();
        let mut octave = 4;
        let mut default_length = 4;
        let mut bpm = 120;
        // デフォルト値V10（BR-074準拠）
        let mut current_velocity: u8 = 10;

        for command in &mml.commands {
            match command {
                Command::Note(note) => {
                    let note_samples = self.generate_note_samples(
                        note,
                        octave,
                        bpm,
                        default_length,
                        current_velocity,
                    );
                    samples.extend(note_samples);
                }
                Command::Rest(rest) => {
                    let duration = rest.duration_in_seconds(bpm, default_length);
                    let num_samples = (f64::from(duration) * f64::from(self.sample_rate)) as usize;
                    samples.extend(vec![0.0; num_samples]);
                }
                Command::Octave(o) => octave = o.value,
                Command::OctaveUp => octave = octave.saturating_add(1).min(8),
                Command::OctaveDown => octave = octave.saturating_sub(1).max(1),
                Command::Tempo(t) => bpm = t.value,
                Command::DefaultLength(l) => default_length = l.value,
                Command::Volume(v) => {
                    current_velocity = match v.value {
                        VolumeValue::Absolute(val) => val,
                        VolumeValue::Relative(delta) => {
                            // 現在値に加算/減算し、0-15にクランプ
                            #[allow(clippy::cast_possible_wrap)]
                            let new_val = (current_velocity as i8 + delta).clamp(0, 15);
                            #[allow(clippy::cast_sign_loss)]
                            {
                                new_val as u8
                            }
                        }
                    };
                }
                Command::Loop { .. } => {
                    unreachable!("Loop commands should be expanded before synthesis")
                }
                Command::Tuplet {
                    commands: tuplet_commands,
                    count,
                    base_duration,
                } => {
                    let tuplet_samples = self.synthesize_tuplet(
                        tuplet_commands,
                        *count,
                        *base_duration,
                        &mut octave,
                        bpm,
                        default_length,
                        current_velocity,
                    );
                    samples.extend(tuplet_samples);
                }
            }
        }

        // Normalization (F-019)
        normalize_samples(&mut samples);

        Ok(samples)
    }

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    fn generate_note_samples(
        &self,
        note: &Note,
        octave: u8,
        bpm: u16,
        default_length: u8,
        velocity: u8,
    ) -> Vec<f32> {
        let midi_note = note.to_midi_note(octave);
        let frequency = midi_to_frequency(midi_note);
        let duration = note.duration_in_seconds(bpm, default_length);
        let num_samples = (f64::from(duration) * f64::from(self.sample_rate)) as usize;

        let mut audio_node = create_node(self.waveform_type, frequency);
        audio_node.set_sample_rate(f64::from(self.sample_rate));

        // velocityは0-15の範囲、15で最大音量
        let master_gain = (f32::from(self.volume) / 100.0) * (f32::from(velocity) / 15.0);

        let mut samples = Vec::with_capacity(num_samples);
        for _ in 0..num_samples {
            let sample = audio_node.get_mono() as f32;
            samples.push(sample * master_gain);
        }

        self.apply_envelope(&mut samples);

        samples
    }

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    fn apply_envelope(&self, samples: &mut [f32]) {
        let fade_duration = 0.005; // 5ms fade
        let fade_samples = (fade_duration * f64::from(self.sample_rate)) as usize;
        let len = samples.len();

        if len == 0 {
            return;
        }

        if len < fade_samples * 2 {
            let half = len / 2;
            for (i, sample) in samples.iter_mut().enumerate().take(half) {
                let gain = i as f32 / half as f32;
                *sample *= gain;
            }
            for (i, sample) in samples.iter_mut().rev().enumerate().take(half) {
                let gain = i as f32 / half as f32;
                *sample *= gain;
            }
            return;
        }

        for (i, sample) in samples.iter_mut().enumerate().take(fade_samples) {
            let gain = i as f32 / fade_samples as f32;
            *sample *= gain;
        }

        // Fade out
        for i in 0..fade_samples {
            let gain = i as f32 / fade_samples as f32;
            samples[len - 1 - i] *= gain;
        }
    }

    #[must_use]
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    pub fn generate_click_samples(&self, _bpm: u16) -> Vec<f32> {
        // Use new noise-based click generation
        generate_noise_click(f64::from(self.sample_rate), f32::from(self.volume) / 100.0)
    }

    /// メトロノームサンプルを演奏サンプルにミックス
    ///
    /// 指定されたビート間隔でクリックサンプルを生成し、演奏サンプルに加算ミックスする。
    /// クリック位置は演奏の先頭から等間隔で配置される。
    ///
    /// # Arguments
    /// * `samples` - 演奏サンプル（可変参照、この関数でクリックが加算される）
    /// * `sample_rate` - サンプリングレート（Hz）
    /// * `bpm` - テンポ（BPM）
    /// * `beat` - ビート値（4, 8, 16）
    /// * `volume` - メトロノーム音量（0.0〜1.0）
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    pub fn mix_metronome(
        &self,
        samples: &mut [f32],
        sample_rate: f64,
        bpm: u16,
        beat: u8,
        volume: f32,
    ) {
        // 1. クリック間隔計算
        let interval_sec = beat_interval_seconds(bpm, beat);
        let interval_samples = (interval_sec * sample_rate as f32) as usize;

        // 2. クリックサンプル生成
        let click_samples = generate_noise_click(sample_rate, volume);

        // 3. ミックス処理
        let mut position = 0;
        while position < samples.len() {
            // クリックサンプルを加算ミックス
            for (i, &click_sample) in click_samples.iter().enumerate() {
                let sample_index = position + i;
                if sample_index >= samples.len() {
                    break;
                }
                samples[sample_index] += click_sample;
            }

            // 次のクリック位置へ
            position += interval_samples;

            // 無限ループ防止
            if interval_samples == 0 {
                break;
            }
        }
    }

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::too_many_arguments
    )]
    fn synthesize_tuplet(
        &self,
        commands: &[Command],
        count: u8,
        base_duration: Option<u8>,
        octave: &mut u8,
        bpm: u16,
        default_length: u8,
        velocity: u8,
    ) -> Vec<f32> {
        let mut samples = Vec::new();

        let base = base_duration.unwrap_or(default_length);
        let base_seconds = 60.0 / f32::from(bpm) * 4.0 / f32::from(base);
        let tuplet_duration = base_seconds / f32::from(count);

        for cmd in commands {
            match cmd {
                Command::Note(note) => {
                    let note_duration =
                        if note.duration.base.value.is_some() || note.duration.has_ties() {
                            note.duration_in_seconds(bpm, default_length) / f32::from(count)
                        } else {
                            tuplet_duration
                        };

                    let midi_note = note.to_midi_note(*octave);
                    let frequency = midi_to_frequency(midi_note);
                    let num_samples =
                        (f64::from(note_duration) * f64::from(self.sample_rate)) as usize;

                    let mut audio_node = create_node(self.waveform_type, frequency);
                    audio_node.set_sample_rate(f64::from(self.sample_rate));

                    let master_gain =
                        (f32::from(self.volume) / 100.0) * (f32::from(velocity) / 15.0);

                    let mut note_samples = Vec::with_capacity(num_samples);
                    for _ in 0..num_samples {
                        let sample = audio_node.get_mono() as f32;
                        note_samples.push(sample * master_gain);
                    }

                    self.apply_envelope(&mut note_samples);
                    samples.extend(note_samples);
                }
                Command::Rest(rest) => {
                    let rest_duration =
                        if rest.duration.base.value.is_some() || rest.duration.has_ties() {
                            rest.duration_in_seconds(bpm, default_length) / f32::from(count)
                        } else {
                            tuplet_duration
                        };

                    let num_samples =
                        (f64::from(rest_duration) * f64::from(self.sample_rate)) as usize;
                    samples.extend(vec![0.0; num_samples]);
                }
                Command::Octave(o) => *octave = o.value,
                Command::OctaveUp => *octave = octave.saturating_add(1).min(8),
                Command::OctaveDown => *octave = octave.saturating_sub(1).max(1),
                Command::Tuplet {
                    commands: inner_commands,
                    count: inner_count,
                    base_duration: inner_base,
                } => {
                    let nested_base = if inner_base.is_some() {
                        *inner_base
                    } else {
                        Some(default_length)
                    };

                    let nested_samples = self.synthesize_tuplet(
                        inner_commands,
                        *inner_count,
                        nested_base,
                        octave,
                        bpm,
                        default_length,
                        velocity,
                    );

                    let nested_total_duration =
                        nested_samples.len() as f32 / self.sample_rate as f32;
                    if nested_total_duration > 0.0 {
                        let target_samples =
                            (f64::from(tuplet_duration) * f64::from(self.sample_rate)) as usize;
                        let resampled = resample_linear(&nested_samples, target_samples);
                        samples.extend(resampled);
                    }
                }
                Command::Loop { .. } => {
                    unreachable!("Loop commands should be expanded before synthesis")
                }
                _ => {}
            }
        }

        samples
    }
}

/// Linearly resample audio samples to a target length.
#[must_use]
pub fn resample_linear(samples: &[f32], target_len: usize) -> Vec<f32> {
    if samples.is_empty() || target_len == 0 {
        return vec![0.0; target_len];
    }

    let mut resampled = Vec::with_capacity(target_len);
    let ratio = (samples.len() - 1) as f32 / (target_len - 1).max(1) as f32;

    for i in 0..target_len {
        let pos = i as f32 * ratio;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let idx = pos as usize;
        let frac = pos - idx as f32;

        let sample = if idx + 1 < samples.len() {
            samples[idx] * (1.0 - frac) + samples[idx + 1] * frac
        } else {
            samples[idx]
        };

        resampled.push(sample);
    }

    resampled
}

/// PCMサンプルをノーマライズ（最大絶対値を1.0以下に制限）
///
/// 最大絶対値が1.0を超える場合のみ、全サンプルを比例縮小する。
/// 1.0以下の場合は何もしない（音量を上げない）。
pub fn normalize_samples(samples: &mut [f32]) {
    if samples.is_empty() {
        return;
    }

    let max_abs = samples.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);

    if max_abs <= 1.0 {
        return;
    }

    let scale = 1.0 / max_abs;

    for s in samples {
        *s *= scale;
    }
}

/// ビート値から1クリックあたりの秒数を計算
///
/// # Arguments
/// * `bpm` - テンポ（BPM: Beats Per Minute）、30〜300の範囲
/// * `beat` - ビート値（4, 8, 16のみ有効）
///
/// # Returns
/// クリック間隔（秒）
///
/// # Panics
/// `beat`が4, 8, 16以外の場合にパニックします。
/// ただし、CLIレベルでclapによりバリデーション済みのため、実行時には発生しません。
#[must_use]
pub fn beat_interval_seconds(bpm: u16, beat: u8) -> f32 {
    match beat {
        4 => 60.0 / f32::from(bpm),  // 4分音符: 1拍あたりの秒数
        8 => 30.0 / f32::from(bpm),  // 8分音符: 0.5拍あたりの秒数
        16 => 15.0 / f32::from(bpm), // 16分音符: 0.25拍あたりの秒数
        _ => unreachable!("beat value is validated by clap"),
    }
}

/// ノイズベースのクリックサンプルを生成
///
/// fundspの`noise()`関数によりホワイトノイズを生成し、
/// ハイパスフィルター（5kHz）と指数減衰エンベロープを適用して、
/// ドラムのハイハット風のクリック音を作成する。
///
/// # Arguments
/// * `sample_rate` - サンプリングレート（Hz）通常は44100.0
/// * `volume` - 音量係数（0.0〜1.0）
///
/// # Returns
/// 25msのクリック音サンプル配列（約1102サンプル @44100Hz）
#[must_use]
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn generate_noise_click(sample_rate: f64, volume: f32) -> Vec<f32> {
    const CLICK_DURATION: f64 = 0.025; // 25ms
    const DECAY_RATE: f64 = -10.0;
    const HIGHPASS_CUTOFF: f32 = 5000.0;
    const HIGHPASS_Q: f32 = 1.0;

    let num_samples = (sample_rate * CLICK_DURATION) as usize;
    let mut dsp_graph = noise() >> highpass_hz(HIGHPASS_CUTOFF, HIGHPASS_Q);
    dsp_graph.reset();
    dsp_graph.set_sample_rate(sample_rate);

    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = (i as f64) / sample_rate;
        let noise_sample = dsp_graph.get_mono() as f32;

        // 指数減衰エンベロープ
        let envelope = (DECAY_RATE * t / CLICK_DURATION).exp() as f32;

        // 音量適用
        let final_sample = noise_sample * envelope * volume;
        samples.push(final_sample);
    }
    samples
}
