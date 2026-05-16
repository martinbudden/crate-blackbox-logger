#![doc = include_str!("../README.md")]
//#![no_std]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![warn(unused_results)]
#![warn(clippy::pedantic)]
#![warn(clippy::doc_paragraphs_missing_punctuation)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]

mod blackbox;
mod data;
pub mod drivers;
mod encoding;
pub mod features;
mod field_arrays;
mod field_definitions;
mod log_frames;
mod log_headers;
mod logger;
mod messages;
pub mod state_machine;

pub use crate::drivers::sd_card;

pub use blackbox::{Blackbox, BlackboxConfig, BlackboxDevice, BlackboxMode, BlackboxStartParameters};
pub use data::Event;
pub use encoding::{BlackboxWriter, SliceWriter};
pub use features::Features;
pub use field_definitions::FieldSelect;
pub use logger::Logger;
pub use messages::{GpsMessage, GyroPidMessage, SetpointMessage};
pub use state_machine::StateMachine;
