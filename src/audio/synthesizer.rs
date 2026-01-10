use crate::audio::waveform::{create_node, midi_to_frequency, WaveformType};
use crate::mml::{Command, Mml, Note};
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
                Command::Tempo(t) => bpm = t.value,
                Command::DefaultLength(l) => default_length = l.value,
                Command::Volume(v) => current_velocity = v.value,
            }
        }

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
        let frequency = 1000.0;
        let duration = 0.05; // 50ms
        let num_samples = (duration * f64::from(self.sample_rate)) as usize;
        let mut audio_node = create_node(WaveformType::Sine, frequency);
        audio_node.set_sample_rate(f64::from(self.sample_rate));

        let mut samples = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            let mut sample = audio_node.get_mono() as f32;
            // Linear decay
            let env = 1.0 - (i as f32 / num_samples as f32);
            sample *= env * (f32::from(self.volume) / 100.0);
            samples.push(sample);
        }
        samples
    }
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
}
