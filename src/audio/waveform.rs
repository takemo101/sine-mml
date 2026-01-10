#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waveform_types_exist() {
        // This test will fail to compile if WaveformType or its variants don't exist
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
}
