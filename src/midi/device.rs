//! MIDI device management
//!
//! This module handles MIDI device enumeration and connection.

use midir::{MidiOutput, MidiOutputConnection};

use super::error::MidiError;

/// Information about a MIDI device.
#[derive(Debug, Clone)]
pub struct MidiDeviceInfo {
    /// Device ID (0-indexed)
    pub id: usize,
    /// Device name
    pub name: String,
}

impl MidiDeviceInfo {
    /// Create a new `MidiDeviceInfo`.
    #[must_use]
    pub fn new(id: usize, name: String) -> Self {
        Self { id, name }
    }
}

/// List all available MIDI output devices.
///
/// # Returns
///
/// A vector of device names on success.
///
/// # Errors
///
/// Returns `MidiError::NoDeviceFound` if MIDI output cannot be initialized.
///
/// # Example
///
/// ```no_run
/// use sine_mml::midi::list_midi_devices;
///
/// let devices = list_midi_devices().expect("Failed to list devices");
/// for (i, name) in devices.iter().enumerate() {
///     println!("{}: {}", i, name);
/// }
/// ```
pub fn list_midi_devices() -> Result<Vec<String>, MidiError> {
    let midi_out = MidiOutput::new("sine-mml-list").map_err(|_| MidiError::NoDeviceFound)?;

    let ports = midi_out.ports();
    let mut devices = Vec::new();

    for port in &ports {
        if let Ok(name) = midi_out.port_name(port) {
            devices.push(name);
        }
    }

    Ok(devices)
}

/// List all available MIDI output devices with full info.
///
/// # Returns
///
/// A vector of `MidiDeviceInfo` structs on success.
///
/// # Errors
///
/// Returns `MidiError::NoDeviceFound` if MIDI output cannot be initialized.
pub fn list_midi_devices_info() -> Result<Vec<MidiDeviceInfo>, MidiError> {
    let devices = list_midi_devices()?;
    Ok(devices
        .into_iter()
        .enumerate()
        .map(|(id, name)| MidiDeviceInfo::new(id, name))
        .collect())
}

/// Resolve a device identifier (ID or name) to a device ID.
///
/// The identifier can be:
/// - A numeric string (e.g., "0", "1") - interpreted as device ID
/// - A device name or substring - first matching device is returned
///
/// # Arguments
///
/// * `name_or_id` - Device identifier (numeric ID or name substring)
///
/// # Returns
///
/// The device ID (0-indexed) on success.
///
/// # Errors
///
/// - `MidiError::InvalidDeviceId` if numeric ID is out of range
/// - `MidiError::NoDeviceFound` if name doesn't match any device
///
/// # Example
///
/// ```no_run
/// use sine_mml::midi::resolve_device_id;
///
/// // By ID
/// let id = resolve_device_id("0").expect("Device not found");
///
/// // By name substring
/// let id = resolve_device_id("IAC").expect("Device not found");
/// ```
pub fn resolve_device_id(name_or_id: &str) -> Result<usize, MidiError> {
    // Try to parse as numeric ID first
    if let Ok(id) = name_or_id.parse::<usize>() {
        let devices = list_midi_devices()?;
        if id >= devices.len() {
            return Err(MidiError::InvalidDeviceId { id });
        }
        return Ok(id);
    }

    // Otherwise, search by name (partial match)
    let devices = list_midi_devices()?;
    for (i, device_name) in devices.iter().enumerate() {
        if device_name.contains(name_or_id) {
            return Ok(i);
        }
    }

    Err(MidiError::NoDeviceFound)
}

/// Connect to a MIDI output device.
///
/// # Arguments
///
/// * `device_id_or_name` - Device identifier (numeric ID or name substring)
///
/// # Returns
///
/// A `MidiOutputConnection` on success.
///
/// # Errors
///
/// - `MidiError::InvalidDeviceId` if numeric ID is out of range
/// - `MidiError::NoDeviceFound` if name doesn't match any device or MIDI init fails
/// - `MidiError::ConnectionFailed` if connection to the device fails
///
/// # Example
///
/// ```no_run
/// use sine_mml::midi::connect_midi_device;
///
/// let conn = connect_midi_device("0").expect("Failed to connect");
/// // Use conn to send MIDI messages...
/// ```
pub fn connect_midi_device(device_id_or_name: &str) -> Result<MidiOutputConnection, MidiError> {
    let id = resolve_device_id(device_id_or_name)?;

    let midi_out = MidiOutput::new("sine-mml-output").map_err(|_| MidiError::NoDeviceFound)?;

    let ports = midi_out.ports();
    let port = ports.get(id).ok_or(MidiError::InvalidDeviceId { id })?;

    midi_out
        .connect(port, "sine-mml-output")
        .map_err(|e| MidiError::ConnectionFailed {
            reason: e.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_device_info_new() {
        let info = MidiDeviceInfo::new(0, "Test Device".to_string());
        assert_eq!(info.id, 0);
        assert_eq!(info.name, "Test Device");
    }

    #[test]
    fn test_midi_device_info_clone() {
        let info = MidiDeviceInfo::new(1, "Clone Test".to_string());
        let cloned = info.clone();
        assert_eq!(cloned.id, 1);
        assert_eq!(cloned.name, "Clone Test");
    }

    #[test]
    fn test_midi_device_info_debug() {
        let info = MidiDeviceInfo::new(2, "Debug Test".to_string());
        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("MidiDeviceInfo"));
        assert!(debug_str.contains('2'));
        assert!(debug_str.contains("Debug Test"));
    }

    #[test]
    fn test_list_midi_devices_returns_result() {
        // This test verifies that list_midi_devices() doesn't panic
        // In CI/container environments without MIDI hardware, it may return Err
        let result = list_midi_devices();
        // Either Ok with a list (possibly empty) or Err (no MIDI subsystem)
        match result {
            Ok(devices) => {
                // Success case: verify it returns a Vec (can be empty)
                // Just accessing .len() verifies it's a valid Vec
                let _ = devices.len();
            }
            Err(e) => {
                // No MIDI subsystem available (expected in CI)
                assert!(matches!(e, MidiError::NoDeviceFound));
            }
        }
    }

    #[test]
    fn test_list_midi_devices_info_returns_result() {
        let result = list_midi_devices_info();

        match result {
            Ok(devices) => {
                // Verify that IDs are sequential starting from 0
                for (expected_id, info) in devices.iter().enumerate() {
                    assert_eq!(info.id, expected_id);
                }
            }
            Err(e) => {
                // No MIDI subsystem available (expected in CI)
                assert!(matches!(e, MidiError::NoDeviceFound));
            }
        }
    }

    #[test]
    fn test_resolve_device_id_invalid_numeric() {
        // A very large ID should fail with either NoDeviceFound or InvalidDeviceId
        let result = resolve_device_id("99999");
        assert!(result.is_err());

        match result.unwrap_err() {
            MidiError::InvalidDeviceId { id } => {
                // MIDI subsystem available but ID out of range
                assert_eq!(id, 99999);
            }
            MidiError::NoDeviceFound => {
                // No MIDI subsystem available (expected in CI)
            }
            other => panic!("Unexpected error: {:?}", other),
        }
    }

    #[test]
    fn test_resolve_device_id_nonexistent_name() {
        // A name that doesn't exist should fail
        let result = resolve_device_id("NonExistentDeviceName12345");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MidiError::NoDeviceFound));
    }

    #[test]
    fn test_connect_midi_device_invalid_id() {
        // A very large ID should fail
        let result = connect_midi_device("99999");
        assert!(result.is_err());
    }

    #[test]
    fn test_connect_midi_device_nonexistent_name() {
        // A name that doesn't exist should fail
        let result = connect_midi_device("NonExistentDeviceName12345");
        assert!(result.is_err());
    }

    // Integration test: only runs when MIDI devices are available
    #[test]
    #[ignore = "requires MIDI hardware - run with `cargo test -- --ignored`"]
    fn test_resolve_device_id_with_real_device() {
        let devices = list_midi_devices().expect("Failed to list devices");
        if !devices.is_empty() {
            // Test numeric ID resolution
            let id = resolve_device_id("0").expect("Failed to resolve ID 0");
            assert_eq!(id, 0);

            // Test name resolution with first device
            let first_name = &devices[0];
            let id_by_name = resolve_device_id(first_name).expect("Failed to resolve by name");
            assert_eq!(id_by_name, 0);
        }
    }

    #[test]
    #[ignore = "requires MIDI hardware - run with `cargo test -- --ignored`"]
    fn test_connect_midi_device_with_real_device() {
        let devices = list_midi_devices().expect("Failed to list devices");
        if !devices.is_empty() {
            let conn = connect_midi_device("0").expect("Failed to connect to device 0");
            // Connection successful, conn will be dropped automatically
            drop(conn);
        }
    }
}
