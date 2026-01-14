//! MIDI module for sine-mml
//!
//! This module provides MIDI output functionality for the MML synthesizer.
//! It is gated behind the `midi-output` feature flag.

pub mod device;
pub mod error;
pub mod message;
pub mod player;

pub use device::*;
pub use error::*;
pub use message::*;
pub use player::*;
