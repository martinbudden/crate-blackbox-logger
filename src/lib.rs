#![doc = include_str!("../README.md")]
#![cfg_attr(not(test), no_std)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
//#![deny(missing_docs)]
#![deny(
    missing_copy_implementations,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_must_use,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]
#![warn(unused_results)]
#![warn(clippy::pedantic)]
#![warn(clippy::doc_paragraphs_missing_punctuation)]

mod blackbox;
mod data;
mod encoding;
mod field_arrays;
mod field_definitions;
mod log_frames;
mod logger;
mod logger_state;
mod write_headers;

pub use blackbox::{Blackbox, BlackboxConfig, BlackboxDevice, BlackboxMode, BlackboxStartParameters};
pub use data::{BlackboxEvent, BlackboxGpsData, BlackboxGpsPosition, BlackboxMainData, BlackboxSlowData};
pub use encoding::{BlackboxWriter, SliceEncoder};
pub use logger::Logger;
pub use logger_state::LoggerState;
