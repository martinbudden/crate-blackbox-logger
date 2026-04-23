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
mod blackbox_context;
mod blackbox_encoding;
mod blackbox_field_arrays;
mod blackbox_field_definitions;
mod blackbox_headers;
mod blackbox_log_frames;
mod blackbox_states;
mod blackbox_telemetry;
pub mod drivers;
pub mod features;

pub use crate::drivers::sd_card;
pub use features::Features;

pub use blackbox::{Blackbox, BlackboxConfig, BlackboxDevice, BlackboxMode, BlackboxStartParameters};
pub use blackbox_encoding::{BlackboxWriter, SliceWriter};
pub use blackbox_telemetry::{BlackboxSlowTelemetry, BlackboxTelemetry};
