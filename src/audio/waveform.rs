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

pub fn generate_sine(freq: f32) -> Box<dyn AudioUnit> {
    Box::new(sine_hz(freq))
}

pub fn generate_sawtooth(freq: f32) -> Box<dyn AudioUnit> {
    Box::new(saw_hz(freq))
}

pub fn generate_square(freq: f32) -> Box<dyn AudioUnit> {
    Box::new(square_hz(freq))
}

pub fn create_node(waveform: WaveformType, freq: f32) -> Box<dyn AudioUnit> {
    match waveform {
        WaveformType::Sine => generate_sine(freq),
        WaveformType::Sawtooth => generate_sawtooth(freq),
        WaveformType::Square => generate_square(freq),
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
        // Just verify it doesn't panic and returns a valid Box
        let _node = create_node(WaveformType::Sine, 440.0);
        let _node2 = create_node(WaveformType::Sawtooth, 440.0);
        let _node3 = create_node(WaveformType::Square, 440.0);
    }
}
