#![doc = include_str!("../README.md")]
//#![no_std]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![warn(unused_results)]
#![warn(clippy::pedantic)]
#![warn(clippy::doc_paragraphs_missing_punctuation)]
#![allow(clippy::inline_always)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]

mod blackbox;
mod blackbox_field_definitions;
mod blackbox_field_arrays;
mod blackbox_encoding;
mod blackbox_headers;

pub use blackbox::{Blackbox};
pub use blackbox_field_definitions::{SimpleFieldDefinition,ConditionalFieldDefinition,MainFieldDefinition};
pub use blackbox_field_definitions::{FieldSign,FieldCondition,FieldEncoding,FieldPredictor};
pub use blackbox_field_arrays::{BLACKBOX_SLOW_FIELDS,BLACKBOX_GPS_G_FIELDS,BLACKBOX_GPS_H_FIELDS,BLACKBOX_MAIN_FIELDS};
pub use blackbox_encoding::{BlackboxBuffer,SliceWriter};
pub use blackbox_headers::{HeaderWriter,write_simple_field_headers,write_main_headers,write_p_interval_header,write_conditional_headers};