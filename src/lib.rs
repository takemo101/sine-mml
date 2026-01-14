#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::float_cmp)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::no_effect_underscore_binding)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
pub mod audio;
pub mod cli;
pub mod db;
#[cfg(feature = "midi-output")]
pub mod midi;
pub mod mml;
