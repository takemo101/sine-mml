use fundsp::hacker::{saw_hz, sine_hz, square_hz, AudioUnit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Sine,
    Sawtooth,
    Square,
}

// Alias to satisfy the requirement if needed, or just use Type.
pub type WaveformType = Type;

#[must_use]
pub fn midi_to_frequency(note: u8) -> f32 {
    440.0 * 2.0f32.powf((f32::from(note) - 69.0) / 12.0)
}

#[must_use]
pub fn generate_sine(freq: f32) -> Box<dyn AudioUnit> {
    Box::new(sine_hz(freq))
}

#[must_use]
pub fn generate_sawtooth(freq: f32) -> Box<dyn AudioUnit> {
    Box::new(saw_hz(freq))
}

#[must_use]
pub fn generate_square(freq: f32) -> Box<dyn AudioUnit> {
    Box::new(square_hz(freq))
}

#[must_use]
pub fn create_node(waveform: Type, freq: f32) -> Box<dyn AudioUnit> {
    match waveform {
        Type::Sine => generate_sine(freq),
        Type::Sawtooth => generate_sawtooth(freq),
        Type::Square => generate_square(freq),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waveform_types_exist() {
        let _sine = Type::Sine;
        let _saw = Type::Sawtooth;
        let _sq = Type::Square;
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
        let _node = create_node(Type::Sine, 440.0);
    }
}
