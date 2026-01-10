use fundsp::hacker::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveformType {
    Sine,
    Sawtooth,
    Square,
}

pub fn midi_to_frequency(note: u8) -> f32 {
    440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0)
}

// fundsp 0.18: AudioUnit is the trait. AudioUnit64 is a typedef for Box<dyn AudioUnit>.
// If AudioUnit64 is not found, we use Box<dyn AudioUnit>.
pub fn create_node(waveform: WaveformType, freq: f32) -> Box<dyn AudioUnit> {
    let freq = freq as f64;
    match waveform {
        WaveformType::Sine => Box::new(sine_hz(freq)),
        WaveformType::Sawtooth => Box::new(saw_hz(freq)),
        WaveformType::Square => Box::new(square_hz(freq)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waveform_types_exist() {
        let _sine = WaveformType::Sine;
        let _saw = WaveformType::Sawtooth;
        let _sq = WaveformType::Square;
    }

    #[test]
    fn test_midi_to_frequency() {
        // A4 = 69 -> 440.0 Hz
        assert!((midi_to_frequency(69) - 440.0).abs() < 1e-6);
        // A3 = 57 -> 220.0 Hz
        assert!((midi_to_frequency(57) - 220.0).abs() < 1e-6);
        // C4 = 60 -> 261.63 Hz
        assert!((midi_to_frequency(60) - 261.62558).abs() < 1e-3);
    }
    
    #[test]
    fn test_create_node() {
        // Just verify it doesn't panic
        let _node = create_node(WaveformType::Sine, 440.0);
    }
}
