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
mod blackbox_encoding;
mod blackbox_field_arrays;
mod blackbox_field_definitions;
mod blackbox_headers;
mod blackbox_log_frames;
mod blackbox_states;
mod blackbox_telemetry;
pub mod drivers;

pub use crate::drivers::sd_card;

pub use blackbox::{Blackbox, BlackboxConfig, BlackboxDevice, BlackboxMode, BlackboxStart};
pub use blackbox_encoding::{BlackboxWriter, SliceWriter};
pub use blackbox_field_arrays::{
    BLACKBOX_GPS_G_FIELDS, BLACKBOX_GPS_H_FIELDS, BLACKBOX_MAIN_FIELDS, BLACKBOX_SLOW_FIELDS,
};
pub use blackbox_field_definitions::{ConditionalFieldDefinition, MainFieldDefinition, SimpleFieldDefinition};
pub use blackbox_field_definitions::{
    FieldCondition, FieldEncoding, FieldHeader, FieldPredictor, FieldSign, LogFieldSelect,
};
pub use blackbox_headers::{write_conditional_header, write_header, write_main_header, write_simple_header};
pub use blackbox_states::{GpsState, MainState, SlowState};
pub use blackbox_telemetry::BlackboxTelemetry;
