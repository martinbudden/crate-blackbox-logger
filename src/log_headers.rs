use crate::encoding::write_field_line;
use crate::field_definitions::{MainFieldDefinition, SimpleFieldDefinition};
use crate::logger::Logger;
use crate::{BlackboxWriter, SliceWriter};

// order of headers in log file is:
// 1. file header
// 2. main fields header
// 3. slow fields header
// 4. system info
// After the headers we have the logging data.
impl Logger {
    pub fn log_file_header(writer: &mut SliceWriter) -> usize {
        writer.write_h_str("Product:Blackbox flight data recorder by Nicholas Sherlock\n");
        writer.write_h_str("Data version:2\n");
        writer.pos
    }

    // Note: this mini state machine will go once I start using async and await.
    pub fn log_main_fields_header(&mut self, writer: &mut SliceWriter, index: usize) -> usize {
        const MAIN_FIELDS: &[MainFieldDefinition] = crate::field_arrays::BLACKBOX_MAIN_FIELDS;

        let filter = |f: &MainFieldDefinition| self.conditions.test(f.condition);

        match index {
            0 => {
                // Name line. Note: This can exceed 500 bytes.
                // Currently using buffer of size 1024, this will be solved once I start using async and await.
                write_field_line(writer, 'I', "name", MAIN_FIELDS.iter().filter(|&f| filter(f)), |w, f| {
                    w.write_str(f.name);
                    let index = f.field_name_index;
                    if index >= 0 {
                        w.write_char('[');
                        w.write_u8_ascii(index.cast_unsigned());
                        w.write_char(']');
                    }
                });
            }
            1 => {
                // I Signed line
                let filtered = MAIN_FIELDS.iter().filter(|&f| filter(f));
                write_field_line(writer, 'I', "signed", filtered, |w, f| {
                    w.write_u8_ascii(f.is_signed);
                });
            }
            2 => {
                // I Predictor line
                let filtered = MAIN_FIELDS.iter().filter(|&f| filter(f));
                write_field_line(writer, 'I', "predictor", filtered, |w, f| {
                    w.write_u8_ascii(f.i_predict);
                });
            }
            3 => {
                // I Encoding line
                let filtered = MAIN_FIELDS.iter().filter(|&f| filter(f));
                write_field_line(writer, 'I', "encoding", filtered, |w, f| {
                    w.write_u8_ascii(f.i_encode);
                });
            }
            4 => {
                // P Predictor line
                let filtered = MAIN_FIELDS.iter().filter(|&f| filter(f));
                write_field_line(writer, 'P', "predictor", filtered, |w, f| {
                    w.write_u8_ascii(f.p_predict);
                });
            }
            5 => {
                // P Encoding line
                let filtered = MAIN_FIELDS.iter().filter(|&f| filter(f));
                write_field_line(writer, 'P', "encoding", filtered, |w, f| {
                    w.write_u8_ascii(f.p_encode);
                });
            }
            _ => {
                return 0;
            }
        }

        writer.pos
    }

    #[allow(clippy::unused_self)]
    pub fn log_slow_fields_header(&mut self, writer: &mut SliceWriter) -> usize {
        const SLOW_FIELDS: &[SimpleFieldDefinition; crate::field_arrays::SLOW_FIELD_COUNT] =
            &crate::field_arrays::BLACKBOX_SLOW_FIELDS;
        let filter = |_: &SimpleFieldDefinition| true;

        // Name line.
        write_field_line(writer, 'S', "name", SLOW_FIELDS.iter().filter(|&f| filter(f)), |w, f| {
            w.write_str(f.name);
            let index = f.field_name_index;
            if index >= 0 {
                w.write_char('[');
                w.write_u8_ascii(index.cast_unsigned());
                w.write_char(']');
            }
        });
        // Signed line
        let filtered = SLOW_FIELDS.iter().filter(|&f| filter(f));
        write_field_line(writer, 'S', "signed", filtered, |w, f| {
            w.write_u8_ascii(f.is_signed);
        });
        // Predictor line
        let filtered = SLOW_FIELDS.iter().filter(|&f| filter(f));
        write_field_line(writer, 'S', "predictor", filtered, |w, f| {
            w.write_u8_ascii(f.predict);
        });
        // Encoding line
        let filtered = SLOW_FIELDS.iter().filter(|&f| filter(f));
        write_field_line(writer, 'S', "encoding", filtered, |w, f| {
            w.write_u8_ascii(f.encode);
        });

        writer.pos
    }

    // Note: this mini state machine will go once I start using async and await.
    pub fn log_sys_info(&mut self, writer: &mut SliceWriter, index: usize) -> usize {
        match index {
            0 => {
                writer.write_h_str("Firmware type:Cleanflight\n");
            }
            1 => {
                writer.write_h_str("Firmware revision:Betaflight 3.3.1 (611bc70f8) REVOLT\n");
            }
            2 => {
                writer.write_h_str("Firmware date:Mar 21 2018 00:00:00\n");
            }
            3 => {
                writer.write_h_str_u32_ascii("P interval:1/", self.p_interval);
                // "P denom" ignored by blackbox-log-view
                // writer.write_h_str("P denom:32\n");
            }
            4 => {
                writer.write_h_str("Device UID:0x0000\n");
            }
            5 => {
                writer.write_h_str_u32_ascii("rcRate:", 1);
            }
            6 => {
                writer.write_h_str("gyro.scale:0x3f800000\n");
                writer.write_h_str("acc_1G:4096\n");
            }
            7 => {
                writer.write_h_str("minthrottle:1070\n");
                writer.write_h_str("maxthrottle:2000\n");
            }
            8 => {
                writer.write_h_str("vbatscale:110\n");
                writer.write_h_str("vbatcellvoltage:33,35,43\n");
                writer.write_h_str("vbatref:113\n");
                writer.write_h_str("currentMeter:0,235\n");
            }
            9 => {
                writer.write_h_str("features:541130760\n");
                writer.write_h_str("debug_mode:0\n");
            }
            _ => {
                return 0;
            }
        }

        writer.pos
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused_results)]
    #![allow(clippy::unwrap_used)]
    use crate::state_machine::StateMachine;
    use crate::{BlackboxStartParameters, BlackboxTelemetry};

    #[allow(unused)]
    use super::*;

    #[allow(unused)]
    fn is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<Logger>();
    }
    #[test]
    fn new() {
        let ctx = Logger::new();
        assert!(!ctx.logged_any_frames);
    }

    #[test]
    fn log_header() {
        let mut buffer = [0u8; 2048];
        //let mut sd_card = MockSdCard::new("state_machine_log.bbl");
        let pos = {
            let mut writer = SliceWriter { buffer: &mut buffer, pos: 0 };

            _ = Logger::log_file_header(&mut writer);

            // Convert the written portion to a string for validation
            let result = core::str::from_utf8(&writer.buffer[..writer.pos]).unwrap();
            // Print for manual inspection (if running with `cargo test -- --nocapture`)
            println!("\nHEADER\n{result}\n");
            assert!(result.contains("H Product:Blackbox"));
            writer.pos
        };
        let result = core::str::from_utf8(&buffer[..pos]).unwrap();
        println!("\nBUFFER\n{result}\n");

        //sd_card.write_all(&buffer[..pos]);
    }
    #[test]
    fn log_main_fields_header() {
        let mut buffer = [0u8; 2048];
        let mut writer = SliceWriter { buffer: &mut buffer, pos: 0 };
        let mut ctx = Logger::new();
        ctx.init(0);

        let mut index: usize = 0;
        loop {
            if ctx.log_main_fields_header(&mut writer, index) == 0 {
                break;
            }
            index += 1;
        }

        // Convert the written portion to a string for validation
        #[allow(clippy::unwrap_used)]
        let result = core::str::from_utf8(&writer.buffer[..writer.pos]).unwrap();
        // Print for manual inspection (if running with `cargo test -- --nocapture`)
        println!("\nMAIN FIELDS HEADER\n{result}\n");
    }
    #[test]
    fn log_slow_fields_header() {
        let mut buffer = [0u8; 2048];
        let mut writer = SliceWriter { buffer: &mut buffer, pos: 0 };
        let mut ctx = Logger::new();
        ctx.init(0);

        let len = ctx.log_slow_fields_header(&mut writer);
        assert_eq!(writer.pos, len);

        // Convert the written portion to a string for validation
        #[allow(clippy::unwrap_used)]
        let result = core::str::from_utf8(&writer.buffer[..writer.pos]).unwrap();
        // Print for manual inspection (if running with `cargo test -- --nocapture`)
        println!("\nSLOW FIELDS HEADER\n{result}\n");
    }
    #[test]
    fn log_sys_info() {
        let mut buffer = [0u8; 2048];
        let mut writer = SliceWriter { buffer: &mut buffer, pos: 0 };
        let mut ctx = Logger::new();

        let mut index: usize = 0;
        loop {
            if ctx.log_sys_info(&mut writer, index) == 0 {
                break;
            }
            index += 1;
        }

        // Convert the written portion to a string for validation
        #[allow(clippy::unwrap_used)]
        let result = core::str::from_utf8(&writer.buffer[..writer.pos]).unwrap();
        // Print for manual inspection (if running with `cargo test -- --nocapture`)
        println!("\nSYS INFO\n{result}\n");
    }
    #[test]
    fn state_machine_headers() {
        let mut buffer = [0u8; 4096];
        let mut writer = SliceWriter { buffer: &mut buffer, pos: 0 };
        let mut ctx = Logger::new();
        //let mut _sd_card = MockSdCard::new("state_machine_log.bbl");
        ctx.init(0);

        let start = BlackboxStartParameters::new();
        let mut state = StateMachine::default();
        let mut current_time_us: u32 = 0;
        let telemetry = BlackboxTelemetry::new();
        println!("\nSTATE MACHINE HEADERS\n");
        state.start(start);
        loop {
            ctx.load_telemetry(current_time_us, telemetry);
            _ = state.update(&mut ctx, &mut writer, current_time_us);
            if state == StateMachine::Running {
                if writer.pos != 0 {
                    #[allow(clippy::unwrap_used)]
                    let result = core::str::from_utf8(&writer.buffer[..writer.pos]).unwrap();
                    println!("{result}");
                }
                break;
            }
            current_time_us = current_time_us.wrapping_add(1000); // use wrapping_add to handle when time rolls over at max u32.
        }
    }
    /*#[test]
    fn full_run() {
        let mut buffer = [0u8; 4096];
        let mut writer = SliceWriter { buffer: &mut buffer, pos: 0 };
        let mut ctx = Blackbox::new();
        ctx.init(0);

        let start = BlackboxStartParameters::new();
        let mut state = State::default();
        let mut current_time_us: u32 = 0;
        let telemetry = BlackboxTelemetry::new();
        state.start(start);
        let mut run_count = 0;
        loop {
            ctx.load_main_data(current_time_us, telemetry);
            _ = state.update(&mut ctx, &mut writer, current_time_us);
            //let state_i:u32 = state.into();
            //println!("state={state_i}");
            if state == State::Running {
                if writer.pos != 0 {
                    #[allow(clippy::unwrap_used)]
                    let result = core::str::from_utf8(&writer.buffer[..writer.pos]).unwrap();
                    println!("RR__{result}__RR");
                    run_count += 1;
                }
                if run_count > 10 {
                    break;
                }
            }
            current_time_us = current_time_us.wrapping_add(1000); // use wrapping_add to handle when time rolls over at max u32.
        }
    }*/
}
