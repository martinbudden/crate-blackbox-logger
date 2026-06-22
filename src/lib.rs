#![doc = include_str!("../README.md")]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![warn(unused_results)]
#![warn(clippy::pedantic)]
#![warn(clippy::doc_paragraphs_missing_punctuation)]

mod blackbox;
mod data;
mod encoding;
mod field_arrays;
mod field_definitions;
mod log_frames;
mod log_headers;
mod logger;
mod messages;
mod state_machine;

pub use blackbox::{Blackbox, BlackboxConfig, BlackboxDevice, BlackboxMode, BlackboxStartParameters};
pub use data::Event;
pub use encoding::{BlackboxWriter, SliceEncoder};
pub use field_definitions::FieldSelect;
pub use logger::Logger;
pub use messages::{GpsMessage, GyroPidMessage, SetpointMessage};
pub use state_machine::LoggerState;
