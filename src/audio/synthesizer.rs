use crate::audio::waveform::{create_node, midi_to_frequency, WaveformType};
use crate::mml::{Command, Mml, Note};
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
        let mut current_velocity = 100;

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
                Command::Volume(v) => current_velocity = v.value,
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

        let master_gain = (f32::from(self.volume) / 100.0) * (f32::from(velocity) / 100.0);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::waveform::WaveformType;
    use crate::mml::{Accidental, Command, Mml, Note, Pitch, Tempo};

    #[test]
    fn test_synthesizer_creation() {
        let synth = Synthesizer::new(44100, 100, WaveformType::Sine);
        assert_eq!(synth.sample_rate, 44100);
        assert_eq!(synth.volume, 100);
        assert_eq!(synth.waveform_type, WaveformType::Sine);
    }

    #[test]
    fn test_synthesize_simple_note() {
        let mut synth = Synthesizer::new(44100, 100, WaveformType::Sine);
        let note = Note {
            pitch: Pitch::A,
            accidental: Accidental::Natural,
            duration: Some(4), // Quarter note
            dots: 0,
        };
        let mml = Mml {
            commands: vec![Command::Tempo(Tempo { value: 120 }), Command::Note(note)],
        };

        let samples = synth.synthesize(&mml).expect("Synthesize failed");
        // Quarter note at 120 BPM is 0.5s. 44100 * 0.5 = 22050.
        assert!((samples.len() as i32 - 22050).abs() <= 1);

        // Check if samples are not all zero (sine wave)
        let has_signal = samples.iter().any(|&x| x.abs() > 0.001);
        assert!(has_signal, "Samples should contain audio signal");
    }

    #[test]
    fn test_click_generation() {
        let synth = Synthesizer::new(44100, 100, WaveformType::Sine);
        let samples = synth.generate_click_samples(120);
        assert!(!samples.is_empty());
    }

    #[test]
    fn test_normalize_clipping_samples() {
        let mut samples = vec![-1.5, 0.8, 1.2];
        normalize_samples(&mut samples);
        // Expected: [-1.0, 0.533, 0.8]
        // Scale = 1.0 / 1.5 = 0.6666...
        assert!((samples[0] - (-1.0)).abs() < 0.001);
        assert!((samples[1] - 0.5333).abs() < 0.001);
        assert!((samples[2] - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_normalize_no_change_when_within_range() {
        let original = vec![-0.8, 0.5, 0.9];
        let mut samples = original.clone();
        normalize_samples(&mut samples);
        assert_eq!(samples, original);
    }

    #[test]
    fn test_normalize_boundary_case() {
        let mut samples = vec![1.0, -1.0];
        let original = samples.clone();
        normalize_samples(&mut samples);
        assert_eq!(samples, original);
    }

    #[test]
    fn test_normalize_empty_slice() {
        let mut samples: Vec<f32> = vec![];
        normalize_samples(&mut samples);
        assert!(samples.is_empty());
    }

    #[test]
    fn test_normalize_single_sample_exceeding() {
        let mut samples = vec![2.0];
        normalize_samples(&mut samples);
        assert!((samples[0] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_normalize_single_sample_within_range() {
        let mut samples = vec![0.5];
        normalize_samples(&mut samples);
        assert_eq!(samples[0], 0.5);
    }

    #[test]
    fn test_normalize_all_zeros() {
        let mut samples = vec![0.0, 0.0, 0.0];
        normalize_samples(&mut samples);
        assert_eq!(samples, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_normalize_negative_peak() {
        let mut samples = vec![-2.0, 0.5, -1.8];
        normalize_samples(&mut samples);
        // max abs is 2.0. Scale is 0.5.
        // -2.0 * 0.5 = -1.0
        // 0.5 * 0.5 = 0.25
        // -1.8 * 0.5 = -0.9
        assert!((samples[0] - (-1.0)).abs() < 0.001);
        assert!((samples[1] - 0.25).abs() < 0.001);
        assert!((samples[2] - (-0.9)).abs() < 0.001);
    }

    #[test]
    fn test_noise_click_sample_count() {
        // 25ms at 44100Hz = 1102.5 samples, truncated to 1102
        let samples = generate_noise_click(44100.0, 0.3);
        assert_eq!(samples.len(), 1102);
    }

    #[test]
    fn test_noise_click_volume_application() {
        let samples_low = generate_noise_click(44100.0, 0.1);
        let samples_high = generate_noise_click(44100.0, 0.9);

        let max_low = samples_low.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        let max_high = samples_high.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);

        // Noise is random, but higher volume consistently produces higher peak amplitude
        assert!(
            max_high > max_low * 2.0,
            "High volume should be significantly louder than low volume"
        );
    }

    // beat_interval_seconds tests (Issue #30)
    #[test]
    fn test_beat_interval_4beat_at_60bpm() {
        // 60 BPM, 4ビート: 60.0 / 60 = 1.0秒
        assert!((beat_interval_seconds(60, 4) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_beat_interval_4beat_at_120bpm() {
        // 120 BPM, 4ビート: 60.0 / 120 = 0.5秒
        assert!((beat_interval_seconds(120, 4) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_beat_interval_4beat_at_240bpm() {
        // 240 BPM, 4ビート: 60.0 / 240 = 0.25秒
        assert!((beat_interval_seconds(240, 4) - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_beat_interval_8beat_at_120bpm() {
        // 120 BPM, 8ビート: 30.0 / 120 = 0.25秒
        assert!((beat_interval_seconds(120, 8) - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_beat_interval_16beat_at_120bpm() {
        // 120 BPM, 16ビート: 15.0 / 120 = 0.125秒
        assert!((beat_interval_seconds(120, 16) - 0.125).abs() < 0.001);
    }

    #[test]
    #[should_panic(expected = "validated by clap")]
    fn test_beat_interval_invalid_beat() {
        // 無効なビート値（5）はパニックする
        let _ = beat_interval_seconds(120, 5);
    }
}
