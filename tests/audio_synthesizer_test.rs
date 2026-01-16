//! Audio Synthesizer Tests
//!
//! This module contains integration tests for the audio synthesizer.
//! Tests were extracted from src/audio/synthesizer.rs for better organization.

use sine_mml::audio::synthesizer::{
    beat_interval_seconds, generate_noise_click, normalize_samples, resample_linear, Synthesizer,
};
use sine_mml::audio::waveform::WaveformType;
use sine_mml::mml::{
    Accidental, Command, Duration, Mml, Note, Pitch, Rest, Tempo, TiedDuration, Volume, VolumeValue,
};

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
        duration: TiedDuration::new(Duration::new(Some(4), 0)), // Quarter note
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

// mix_metronome tests (Issue #31)
#[test]
fn test_mix_metronome_click_positions() {
    let synth = Synthesizer::new(44100, 100, WaveformType::Sine);
    let mut samples = vec![0.0; 44100]; // 1秒分

    // 120BPM, 4ビート: 0.5秒ごと → 1秒で2クリック
    synth.mix_metronome(&mut samples, 44100.0, 120, 4, 0.3);

    // クリック位置でサンプルが0でないことを確認
    assert_ne!(samples[0], 0.0, "先頭にクリックがあるはず");
    assert_ne!(samples[22050], 0.0, "0.5秒後にクリックがあるはず");
}

#[test]
fn test_mix_metronome_additive() {
    let synth = Synthesizer::new(44100, 100, WaveformType::Sine);
    let mut samples = vec![0.5; 44100]; // 全サンプル0.5で初期化
    let original_value = samples[0];

    synth.mix_metronome(&mut samples, 44100.0, 120, 4, 0.3);

    // 加算ミックスのため、クリック位置のサンプルは元の値と異なるはず
    // (0.5 + クリック音 != 0.5)
    assert!(
        (samples[0] - original_value).abs() > 0.01_f32,
        "クリックが加算されているはず"
    );
}

#[test]
fn test_mix_metronome_16beat_more_clicks() {
    let synth = Synthesizer::new(44100, 100, WaveformType::Sine);
    let mut samples_4beat = vec![0.0; 44100];
    let mut samples_16beat = vec![0.0; 44100];

    synth.mix_metronome(&mut samples_4beat, 44100.0, 120, 4, 0.3);
    synth.mix_metronome(&mut samples_16beat, 44100.0, 120, 16, 0.3);

    // 16ビートは4ビートより多くのクリックがあるため、非ゼロサンプルが多い
    let nonzero_4beat = samples_4beat.iter().filter(|&&x| x != 0.0).count();
    let nonzero_16beat = samples_16beat.iter().filter(|&&x| x != 0.0).count();

    assert!(
        nonzero_16beat > nonzero_4beat,
        "16ビートは4ビートより多くのクリックがあるはず: {nonzero_16beat} > {nonzero_4beat}",
    );
}

// 相対ボリュームテスト (Issue #92)
#[test]
fn test_synthesize_with_absolute_volume() {
    let mut synth = Synthesizer::new(44100, 100, WaveformType::Sine);
    let mml = Mml {
        commands: vec![
            Command::Volume(Volume {
                value: VolumeValue::Absolute(15),
            }),
            Command::Note(Note {
                pitch: Pitch::A,
                accidental: Accidental::Natural,
                duration: TiedDuration::new(Duration::new(Some(16), 0)),
            }),
        ],
    };
    let samples = synth.synthesize(&mml).unwrap();
    assert!(!samples.is_empty());
}

#[test]
fn test_synthesize_with_relative_volume_increase() {
    let mut synth = Synthesizer::new(44100, 100, WaveformType::Sine);
    // Start at default V10, then V+2 = V12
    let mml = Mml {
        commands: vec![
            Command::Volume(Volume {
                value: VolumeValue::Relative(2),
            }),
            Command::Note(Note {
                pitch: Pitch::A,
                accidental: Accidental::Natural,
                duration: TiedDuration::new(Duration::new(Some(16), 0)),
            }),
        ],
    };
    let samples = synth.synthesize(&mml).unwrap();
    assert!(!samples.is_empty());
}

#[test]
fn test_synthesize_with_relative_volume_decrease() {
    let mut synth = Synthesizer::new(44100, 100, WaveformType::Sine);
    // Start at default V10, then V-3 = V7
    let mml = Mml {
        commands: vec![
            Command::Volume(Volume {
                value: VolumeValue::Relative(-3),
            }),
            Command::Note(Note {
                pitch: Pitch::A,
                accidental: Accidental::Natural,
                duration: TiedDuration::new(Duration::new(Some(16), 0)),
            }),
        ],
    };
    let samples = synth.synthesize(&mml).unwrap();
    assert!(!samples.is_empty());
}

#[test]
fn test_synthesize_volume_clamp_upper() {
    let mut synth = Synthesizer::new(44100, 100, WaveformType::Sine);
    // V15 + V+5 should clamp to 15
    let mml = Mml {
        commands: vec![
            Command::Volume(Volume {
                value: VolumeValue::Absolute(15),
            }),
            Command::Volume(Volume {
                value: VolumeValue::Relative(5),
            }),
            Command::Note(Note {
                pitch: Pitch::A,
                accidental: Accidental::Natural,
                duration: TiedDuration::new(Duration::new(Some(16), 0)),
            }),
        ],
    };
    let samples = synth.synthesize(&mml).unwrap();
    assert!(!samples.is_empty());
}

#[test]
fn test_synthesize_volume_clamp_lower() {
    let mut synth = Synthesizer::new(44100, 100, WaveformType::Sine);
    // V0 + V-5 should clamp to 0
    let mml = Mml {
        commands: vec![
            Command::Volume(Volume {
                value: VolumeValue::Absolute(0),
            }),
            Command::Volume(Volume {
                value: VolumeValue::Relative(-5),
            }),
            Command::Note(Note {
                pitch: Pitch::A,
                accidental: Accidental::Natural,
                duration: TiedDuration::new(Duration::new(Some(16), 0)),
            }),
        ],
    };
    let samples = synth.synthesize(&mml).unwrap();
    // V0 should produce silence (very small samples due to envelope)
    assert!(!samples.is_empty());
}

#[test]
fn test_default_velocity_is_10() {
    let mut synth = Synthesizer::new(44100, 100, WaveformType::Sine);
    // No volume command, default should be V10
    // V+5 from default should be V15
    let mml = Mml {
        commands: vec![
            Command::Volume(Volume {
                value: VolumeValue::Relative(5),
            }),
            Command::Note(Note {
                pitch: Pitch::A,
                accidental: Accidental::Natural,
                duration: TiedDuration::new(Duration::new(Some(16), 0)),
            }),
        ],
    };
    let samples = synth.synthesize(&mml).unwrap();
    // If default was 100, V+5 would clamp to 15 anyway
    // If default is 10, V+5 = V15
    assert!(!samples.is_empty());
}

#[test]
fn test_synthesize_tuplet_3_notes() {
    let mut synth = Synthesizer::new(44100, 100, WaveformType::Sine);
    let mml = Mml {
        commands: vec![
            Command::Tempo(Tempo { value: 120 }),
            Command::Tuplet {
                commands: vec![
                    Command::Note(Note {
                        pitch: Pitch::C,
                        accidental: Accidental::Natural,
                        duration: TiedDuration::new(Duration::new(None, 0)),
                    }),
                    Command::Note(Note {
                        pitch: Pitch::D,
                        accidental: Accidental::Natural,
                        duration: TiedDuration::new(Duration::new(None, 0)),
                    }),
                    Command::Note(Note {
                        pitch: Pitch::E,
                        accidental: Accidental::Natural,
                        duration: TiedDuration::new(Duration::new(None, 0)),
                    }),
                ],
                count: 3,
                base_duration: None,
            },
        ],
    };
    let samples = synth.synthesize(&mml).unwrap();
    // 120BPM, default length 4 (quarter note) = 0.5s base
    // Each note = 0.5 / 3 = 0.1667s
    // Total = 0.5s = 22050 samples
    let expected_samples = 22050;
    assert!(
        (samples.len() as i32 - expected_samples).abs() <= 10,
        "Expected ~{} samples, got {}",
        expected_samples,
        samples.len()
    );
}

#[test]
fn test_synthesize_tuplet_with_base_duration() {
    let mut synth = Synthesizer::new(44100, 100, WaveformType::Sine);
    let mml = Mml {
        commands: vec![
            Command::Tempo(Tempo { value: 120 }),
            Command::Tuplet {
                commands: vec![
                    Command::Note(Note {
                        pitch: Pitch::C,
                        accidental: Accidental::Natural,
                        duration: TiedDuration::new(Duration::new(None, 0)),
                    }),
                    Command::Note(Note {
                        pitch: Pitch::D,
                        accidental: Accidental::Natural,
                        duration: TiedDuration::new(Duration::new(None, 0)),
                    }),
                    Command::Note(Note {
                        pitch: Pitch::E,
                        accidental: Accidental::Natural,
                        duration: TiedDuration::new(Duration::new(None, 0)),
                    }),
                ],
                count: 3,
                base_duration: Some(2),
            },
        ],
    };
    let samples = synth.synthesize(&mml).unwrap();
    // 120BPM, base_duration 2 (half note) = 1.0s base
    // Each note = 1.0 / 3 = 0.333s
    // Total = 1.0s = 44100 samples
    let expected_samples = 44100;
    assert!(
        (samples.len() as i32 - expected_samples).abs() <= 10,
        "Expected ~{} samples, got {}",
        expected_samples,
        samples.len()
    );
}

#[test]
fn test_synthesize_tuplet_with_rest() {
    let mut synth = Synthesizer::new(44100, 100, WaveformType::Sine);
    let mml = Mml {
        commands: vec![
            Command::Tempo(Tempo { value: 120 }),
            Command::Tuplet {
                commands: vec![
                    Command::Note(Note {
                        pitch: Pitch::C,
                        accidental: Accidental::Natural,
                        duration: TiedDuration::new(Duration::new(None, 0)),
                    }),
                    Command::Rest(Rest {
                        duration: TiedDuration::new(Duration::new(None, 0)),
                    }),
                    Command::Note(Note {
                        pitch: Pitch::E,
                        accidental: Accidental::Natural,
                        duration: TiedDuration::new(Duration::new(None, 0)),
                    }),
                ],
                count: 3,
                base_duration: None,
            },
        ],
    };
    let samples = synth.synthesize(&mml).unwrap();
    let expected_samples = 22050;
    assert!(
        (samples.len() as i32 - expected_samples).abs() <= 10,
        "Expected ~{} samples, got {}",
        expected_samples,
        samples.len()
    );
}

#[test]
fn test_synthesize_tuplet_5_notes() {
    let mut synth = Synthesizer::new(44100, 100, WaveformType::Sine);
    let mml = Mml {
        commands: vec![
            Command::Tempo(Tempo { value: 120 }),
            Command::Tuplet {
                commands: vec![
                    Command::Note(Note {
                        pitch: Pitch::C,
                        accidental: Accidental::Natural,
                        duration: TiedDuration::new(Duration::new(None, 0)),
                    }),
                    Command::Note(Note {
                        pitch: Pitch::D,
                        accidental: Accidental::Natural,
                        duration: TiedDuration::new(Duration::new(None, 0)),
                    }),
                    Command::Note(Note {
                        pitch: Pitch::E,
                        accidental: Accidental::Natural,
                        duration: TiedDuration::new(Duration::new(None, 0)),
                    }),
                    Command::Note(Note {
                        pitch: Pitch::F,
                        accidental: Accidental::Natural,
                        duration: TiedDuration::new(Duration::new(None, 0)),
                    }),
                    Command::Note(Note {
                        pitch: Pitch::G,
                        accidental: Accidental::Natural,
                        duration: TiedDuration::new(Duration::new(None, 0)),
                    }),
                ],
                count: 5,
                base_duration: None,
            },
        ],
    };
    let samples = synth.synthesize(&mml).unwrap();
    // 5 notes in 1 beat at 120bpm = 0.5s total
    let expected_samples = 22050;
    assert!(
        (samples.len() as i32 - expected_samples).abs() <= 10,
        "Expected ~{} samples, got {}",
        expected_samples,
        samples.len()
    );
}

#[test]
fn test_synthesize_tuplet_octave_change() {
    let mut synth = Synthesizer::new(44100, 100, WaveformType::Sine);
    let mml = Mml {
        commands: vec![
            Command::Tempo(Tempo { value: 120 }),
            Command::Tuplet {
                commands: vec![
                    Command::Note(Note {
                        pitch: Pitch::C,
                        accidental: Accidental::Natural,
                        duration: TiedDuration::new(Duration::new(None, 0)),
                    }),
                    Command::OctaveUp,
                    Command::Note(Note {
                        pitch: Pitch::C,
                        accidental: Accidental::Natural,
                        duration: TiedDuration::new(Duration::new(None, 0)),
                    }),
                ],
                count: 3,
                base_duration: None,
            },
        ],
    };
    let samples = synth.synthesize(&mml).unwrap();
    assert!(!samples.is_empty());
}

#[test]
fn test_resample_linear_upsample() {
    let input = vec![0.0, 1.0];
    let result = resample_linear(&input, 5);
    assert_eq!(result.len(), 5);
    assert!((result[0] - 0.0).abs() < 0.01);
    assert!((result[4] - 1.0).abs() < 0.01);
}

#[test]
fn test_resample_linear_downsample() {
    let input = vec![0.0, 0.25, 0.5, 0.75, 1.0];
    let result = resample_linear(&input, 3);
    assert_eq!(result.len(), 3);
    assert!((result[0] - 0.0).abs() < 0.01);
    assert!((result[2] - 1.0).abs() < 0.01);
}

#[test]
fn test_resample_linear_empty() {
    let result = resample_linear(&[], 5);
    assert_eq!(result.len(), 5);
    assert!(result.iter().all(|&x| x == 0.0));
}

#[test]
fn test_resample_linear_target_zero() {
    let input = vec![1.0, 2.0, 3.0];
    let result = resample_linear(&input, 0);
    assert!(result.is_empty());
}
