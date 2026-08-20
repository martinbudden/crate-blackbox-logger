use crate::{
    encoding::{write_field_line, write_field_line_header},
    field_definitions::{MainFieldDefinition, SimpleFieldDefinition},
    logger::Logger,
    {BlackboxWriter, SliceEncoder},
};

#[cfg(feature = "gps")]
use crate::field_definitions::ConditionalFieldDefinition;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum FieldHeaderIndex {
    IName(usize),
    ISigned,
    IPredictor,
    IEncoding,
    PPredictor,
    PEncoding,
    End,
}

impl FieldHeaderIndex {
    #[must_use]
    pub const fn new() -> Self {
        Self::IName(0)
    }
}

impl Default for FieldHeaderIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(u8)]
pub enum SysInfoIndex {
    #[default]
    Start = 0,
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    End,
}

impl SysInfoIndex {
    #[must_use]
    pub const fn new() -> Self {
        Self::Start
    }
}

/// The log file has the following structure:
/// 1. file header
/// 2. main fields header
/// 3. slow fields header
/// 4. system info
/// 5. After the headers we have the logging data.
impl Logger {
    /// Write the file header.
    pub fn write_file_header(encoder: &mut SliceEncoder) {
        encoder.write_h_str("Product:Blackbox flight data recorder by Nicholas Sherlock\n");
        encoder.write_h_str("Data version:2\n");
    }

    /// Write the file fields header.
    /// The `FieldHeaderIndex` state machine is used to split the header into chunks,
    /// writing one chunk each iteration so that the encoder buffer does not overflow.
    pub fn write_main_fields_header(
        &mut self,
        encoder: &mut SliceEncoder,
        field_header: FieldHeaderIndex,
    ) -> FieldHeaderIndex {
        const MAIN_FIELDS: &[MainFieldDefinition] = crate::field_arrays::BLACKBOX_MAIN_FIELDS;

        match field_header {
            FieldHeaderIndex::IName(mut name_index) => {
                // write the line header on the first iteration
                if name_index == 0 {
                    write_field_line_header(encoder, 'I', "name");
                }
                // write one field definition each iteration.
                if let Some(f) = MAIN_FIELDS.get(name_index) {
                    name_index += 1; // Move past this item for future evaluations

                    if self.conditions.test(f.condition) {
                        // Process this single field
                        if name_index > 1 {
                            // don't need the comma before the first item.
                            encoder.write_char(',');
                        }
                        encoder.write_str(f.name);
                        if f.name_index >= 0 {
                            encoder.write_char('[');
                            encoder.write_u8_ascii(f.name_index.cast_unsigned());
                            encoder.write_char(']');
                        }
                    }
                    // Return the updated index state to stay in IName
                    return FieldHeaderIndex::IName(name_index);
                }
                encoder.write_char('\n');
                FieldHeaderIndex::ISigned
            }
            FieldHeaderIndex::ISigned => {
                // I Signed line
                let filter = |f: &MainFieldDefinition| self.conditions.test(f.condition);
                let filtered = MAIN_FIELDS.iter().filter(|&f| filter(f));
                write_field_line(encoder, 'I', "signed", filtered, |w, f| {
                    w.write_u8_ascii(f.is_signed as u8);
                });
                FieldHeaderIndex::IPredictor
            }
            FieldHeaderIndex::IPredictor => {
                // I Predictor line
                let filter = |f: &MainFieldDefinition| self.conditions.test(f.condition);
                let filtered = MAIN_FIELDS.iter().filter(|&f| filter(f));
                write_field_line(encoder, 'I', "predictor", filtered, |w, f| {
                    w.write_u8_ascii(f.i_predict as u8);
                });
                FieldHeaderIndex::IEncoding
            }
            FieldHeaderIndex::IEncoding => {
                // I Encoding line
                let filter = |f: &MainFieldDefinition| self.conditions.test(f.condition);
                let filtered = MAIN_FIELDS.iter().filter(|&f| filter(f));
                write_field_line(encoder, 'I', "encoding", filtered, |w, f| {
                    w.write_u8_ascii(f.i_encode as u8);
                });
                FieldHeaderIndex::PPredictor
            }
            FieldHeaderIndex::PPredictor => {
                // P Predictor line
                let filter = |f: &MainFieldDefinition| self.conditions.test(f.condition);
                let filtered = MAIN_FIELDS.iter().filter(|&f| filter(f));
                write_field_line(encoder, 'P', "predictor", filtered, |w, f| {
                    w.write_u8_ascii(f.p_predict as u8);
                });
                FieldHeaderIndex::PEncoding
            }
            FieldHeaderIndex::PEncoding => {
                // P Encoding line
                let filter = |f: &MainFieldDefinition| self.conditions.test(f.condition);
                let filtered = MAIN_FIELDS.iter().filter(|&f| filter(f));
                write_field_line(encoder, 'P', "encoding", filtered, |w, f| {
                    w.write_u8_ascii(f.p_encode as u8);
                });
                FieldHeaderIndex::End
            }
            FieldHeaderIndex::End => FieldHeaderIndex::End,
        }
    }

    /// Write the slow fields header.
    /// It is small enough to be written in one go.
    pub fn write_slow_fields_header(&mut self, encoder: &mut SliceEncoder) {
        const SLOW_FIELDS: &[SimpleFieldDefinition; SimpleFieldDefinition::SLOW_FIELD_COUNT] =
            &crate::field_arrays::BLACKBOX_SLOW_FIELDS;
        let filter = |_: &SimpleFieldDefinition| true;

        // Name line.
        write_field_line(encoder, 'S', "name", SLOW_FIELDS.iter().filter(|&f| filter(f)), |w, f| {
            w.write_str(f.name);
            let index = f.name_index;
            if index >= 0 {
                w.write_char('[');
                w.write_u8_ascii(index.cast_unsigned());
                w.write_char(']');
            }
        });
        // Signed line
        let filtered = SLOW_FIELDS.iter().filter(|&f| filter(f));
        write_field_line(encoder, 'S', "signed", filtered, |w, f| {
            w.write_u8_ascii(f.is_signed as u8);
        });
        // Predictor line
        let filtered = SLOW_FIELDS.iter().filter(|&f| filter(f));
        write_field_line(encoder, 'S', "predictor", filtered, |w, f| {
            w.write_u8_ascii(f.predict as u8);
        });
        // Encoding line
        let filtered = SLOW_FIELDS.iter().filter(|&f| filter(f));
        write_field_line(encoder, 'S', "encoding", filtered, |w, f| {
            w.write_u8_ascii(f.encode as u8);
        });
    }

    #[cfg(feature = "gps")]
    /// Write the gps h fields header.
    /// It is small enough to be written in one go.
    pub fn write_gps_h_fields_header(&mut self, encoder: &mut SliceEncoder) {
        const GPS_H_FIELDS: &[SimpleFieldDefinition; SimpleFieldDefinition::GPS_H_FIELD_COUNT] =
            &crate::field_arrays::BLACKBOX_GPS_H_FIELDS;
        let filter = |_: &SimpleFieldDefinition| true;

        // Name line.
        write_field_line(encoder, 'H', "name", GPS_H_FIELDS.iter().filter(|&f| filter(f)), |w, f| {
            w.write_str(f.name);
            let index = f.name_index;
            if index >= 0 {
                w.write_char('[');
                w.write_u8_ascii(index.cast_unsigned());
                w.write_char(']');
            }
        });
        // Signed line
        let filtered = GPS_H_FIELDS.iter().filter(|&f| filter(f));
        write_field_line(encoder, 'H', "signed", filtered, |w, f| {
            w.write_u8_ascii(f.is_signed as u8);
        });
        // Predictor line
        let filtered = GPS_H_FIELDS.iter().filter(|&f| filter(f));
        write_field_line(encoder, 'H', "predictor", filtered, |w, f| {
            w.write_u8_ascii(f.predict as u8);
        });
        // Encoding line
        let filtered = GPS_H_FIELDS.iter().filter(|&f| filter(f));
        write_field_line(encoder, 'H', "encoding", filtered, |w, f| {
            w.write_u8_ascii(f.encode as u8);
        });
    }

    #[cfg(feature = "gps")]
    /// Write the gps g fields header.
    /// It is small enough to be written in one go.
    pub fn write_gps_g_fields_header(&mut self, encoder: &mut SliceEncoder) {
        const GPS_G_FIELDS: &[ConditionalFieldDefinition; ConditionalFieldDefinition::GPS_G_FIELD_COUNT] =
            &crate::field_arrays::BLACKBOX_GPS_G_FIELDS;
        let filter = |_: &ConditionalFieldDefinition| true;

        // Name line.
        write_field_line(encoder, 'G', "name", GPS_G_FIELDS.iter().filter(|&f| filter(f)), |w, f| {
            w.write_str(f.name);
            let index = f.name_index;
            if index >= 0 {
                w.write_char('[');
                w.write_u8_ascii(index.cast_unsigned());
                w.write_char(']');
            }
        });
        // Signed line
        let filtered = GPS_G_FIELDS.iter().filter(|&f| filter(f));
        write_field_line(encoder, 'G', "signed", filtered, |w, f| {
            w.write_u8_ascii(f.is_signed as u8);
        });
        // Predictor line
        let filtered = GPS_G_FIELDS.iter().filter(|&f| filter(f));
        write_field_line(encoder, 'G', "predictor", filtered, |w, f| {
            w.write_u8_ascii(f.predict as u8);
        });
        // Encoding line
        let filtered = GPS_G_FIELDS.iter().filter(|&f| filter(f));
        write_field_line(encoder, 'G', "encoding", filtered, |w, f| {
            w.write_u8_ascii(f.encode as u8);
        });
    }

    /// Write the system info.
    /// The `SysInfoIndex` state machine is used to split it into chunks,
    /// writing one chunk each iteration so that the encoder buffer does not overflow.
    #[allow(clippy::too_many_lines)]
    pub fn write_sys_info(&mut self, encoder: &mut SliceEncoder, sys_info: SysInfoIndex) -> SysInfoIndex {
        // possibly consider adding the following: rcRate, yawRate, motor_poles
        match sys_info {
            SysInfoIndex::Start => {
                encoder.write_h_str("Firmware type:Betaflight\n");
                // encoder.write_h_str("Firmware revision:Betaflight 4.2.11\n");
                // encoder.write_h_str("Firmware date:Mar 9 2021 00:00:00\n");
                SysInfoIndex::S1
            }
            SysInfoIndex::S1 => {
                // Time in ISO 8601 format: 0000-01-01T00:00:00.000+00:00
                encoder.write_h_str("Log start datetime:");
                encoder.write_u16_ascii(self.sys_info.date_time.year);
                encoder.write_char('-');
                encoder.write_u8_02_ascii(self.sys_info.date_time.month);
                encoder.write_char('-');
                encoder.write_u8_02_ascii(self.sys_info.date_time.day);
                encoder.write_char('T');
                encoder.write_u8_02_ascii(self.sys_info.date_time.hours);
                encoder.write_char(':');
                encoder.write_u8_02_ascii(self.sys_info.date_time.minutes);
                encoder.write_char(':');
                encoder.write_u8_02_ascii(self.sys_info.date_time.seconds);
                encoder.write_str(".000+00:00\n");
                SysInfoIndex::S2
            }
            SysInfoIndex::S2 => {
                encoder.write_h_str_u32_ascii("I interval:", self.i_interval);
                encoder.write_h_str_u32_ascii("P interval:1/", self.p_interval);
                SysInfoIndex::S3
            }
            SysInfoIndex::S3 => {
                encoder.write_h_str_u16_ascii("looptime:", self.sys_info.looptime);
                encoder.write_h_str_u16_ascii("gyro_sync_denom:", self.sys_info.gyro_sync_denom);
                encoder.write_h_str_u16_ascii("pid_process_denom:", self.sys_info.pid_process_denom);
                // "P denom" ignored by blackbox-log-view
                // encoder.write_h_str("P denom:32\n");
                SysInfoIndex::S4
            }
            SysInfoIndex::S4 => {
                encoder.write_h_str_u16_ascii("debug_mode:", self.debug_mode);
                encoder.write_h_str_u32_ascii("features:", self.sys_info.features);
                SysInfoIndex::S5
            }
            SysInfoIndex::S5 => {
                encoder.write_h_str_u32_ascii_hex("gyro_scale:", self.sys_info.gyro_scale);
                encoder.write_h_str_u16_ascii("acc_1G:", self.sys_info.acc_1g);
                SysInfoIndex::S6
            }
            SysInfoIndex::S6 => {
                encoder.write_h_str("motorOutput:");
                encoder.write_u32_ascii(u32::from(self.sys_info.motor_output_min));
                encoder.write_byte(b',');
                encoder.write_u32_ascii(u32::from(self.sys_info.motor_output_max));
                encoder.write_char('\n');

                encoder.write_h_str_u16_ascii("motor_poles:", u16::from(self.sys_info.motor_pole_count));
                SysInfoIndex::S7
            }
            SysInfoIndex::S7 => {
                encoder.write_h_str_u16_ascii("vbatscale:", self.sys_info.vbat_scale);
                encoder.write_h_str("vbatcellvoltage:");
                encoder.write_u16_ascii(self.sys_info.vbat_min_cell_voltage);
                encoder.write_char(',');
                encoder.write_u16_ascii(self.sys_info.vbat_warning_cell_voltage);
                encoder.write_char(',');
                encoder.write_u16_ascii(self.sys_info.vbat_max_cell_voltage);
                encoder.write_char('\n');

                encoder.write_h_str_u16_ascii("vbatref:", self.vbat_reference);
                encoder.write_h_str("currentSensor:");
                encoder.write_u16_ascii(self.sys_info.current_sensor_offset);
                encoder.write_char(',');
                encoder.write_u16_ascii(self.sys_info.current_sensor_scale);
                encoder.write_char('\n');
                SysInfoIndex::End
            }
            SysInfoIndex::End => SysInfoIndex::End,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]
    use crate::{data::BlackboxMainData, logger::BlackboxSysInfo, logger_state::LoggerState};

    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<SysInfoIndex>();
        is_full::<FieldHeaderIndex>();
    }
    #[test]
    fn test_new() {
        let field_header_index = FieldHeaderIndex::new();
        assert_eq!(FieldHeaderIndex::IName(0), field_header_index);
        let sys_info_index = SysInfoIndex::new();
        assert_eq!(SysInfoIndex::Start, sys_info_index);
    }

    #[test]
    fn write_file_header() {
        let mut buffer = [0u8; 2048];
        //let mut sd_card = MockSdCard::new("state_machine_log.bbl");
        let pos = {
            let mut encoder = SliceEncoder { buffer: &mut buffer, pos: 0 };

            Logger::write_file_header(&mut encoder);

            // Convert the written portion to a string for validation
            let Ok(result) = core::str::from_utf8(&encoder.buffer[..encoder.pos]) else {
                panic!("from_utf8 failed");
            };
            // Print for manual inspection (if running with `cargo test -- --nocapture`)
            println!("\nHEADER\n{result}\n");
            assert!(result.contains("H Product:Blackbox"));
            encoder.pos
        };
        let Ok(result) = core::str::from_utf8(&buffer[..pos]) else {
            panic!("from_utf8 failed");
        };
        println!("\nBUFFER\n{result}\n");

        //sd_card.write_all(&buffer[..pos]);
    }
    #[test]
    fn write_main_fields_header() {
        let mut buffer = [0u8; 2048];
        let mut encoder = SliceEncoder { buffer: &mut buffer, pos: 0 };
        let sys_info = BlackboxSysInfo::new();
        let mut logger = Logger::new(sys_info);
        logger.init(0, 0, false);

        let mut field_header: FieldHeaderIndex = FieldHeaderIndex::IName(0);
        loop {
            field_header = logger.write_main_fields_header(&mut encoder, field_header);
            if field_header == FieldHeaderIndex::End {
                break;
            }
        }

        // Convert the written portion to a string for validation
        let Ok(result) = core::str::from_utf8(&encoder.buffer[..encoder.pos]) else {
            panic!("from_utf8 failed");
        };
        // Print for manual inspection (if running with `cargo test -- --nocapture`)
        println!("\nMAIN FIELDS HEADER\n{result}\n");
    }
    #[test]
    fn write_slow_fields_header() {
        let mut buffer = [0u8; 2048];
        let mut encoder = SliceEncoder { buffer: &mut buffer, pos: 0 };
        let sys_info = BlackboxSysInfo::new();
        let mut logger = Logger::new(sys_info);
        logger.init(0, 0, false);

        logger.write_slow_fields_header(&mut encoder);

        // Convert the written portion to a string for validation
        let Ok(result) = core::str::from_utf8(&encoder.buffer[..encoder.pos]) else {
            panic!("from_utf8 failed");
        };
        // Print for manual inspection (if running with `cargo test -- --nocapture`)
        println!("\nSLOW FIELDS HEADER\n{result}\n");
    }
    #[test]
    fn write_sys_info() {
        let mut buffer = [0u8; 2048];
        let mut encoder = SliceEncoder { buffer: &mut buffer, pos: 0 };
        let sys_info = BlackboxSysInfo::new();
        let mut logger = Logger::new(sys_info);

        let mut sys_info = SysInfoIndex::Start;
        loop {
            sys_info = logger.write_sys_info(&mut encoder, sys_info);
            if sys_info == SysInfoIndex::End {
                break;
            }
        }

        // Convert the written portion to a string for validation
        let Ok(result) = core::str::from_utf8(&encoder.buffer[..encoder.pos]) else {
            panic!("from_utf8 failed");
        };
        // Print for manual inspection (if running with `cargo test -- --nocapture`)
        println!("\nSYS INFO\n{result}\n");
    }
    #[test]
    fn state_machine_headers() {
        let mut buffer = [0u8; 4096];
        let mut encoder = SliceEncoder { buffer: &mut buffer, pos: 0 };
        let sys_info = BlackboxSysInfo::new();
        let mut logger = Logger::new(sys_info);
        //let mut _sd_card = MockSdCard::new("state_machine_log.bbl");
        logger.init(0, 0, false);

        let mut state = LoggerState::default();
        assert_eq!(LoggerState::Disabled, state);

        let mut current_time_us: u64 = 0;
        let main_data = BlackboxMainData::new();
        logger.set_main_data(main_data);

        println!("\nSTATE MACHINE HEADERS\n");
        state.start(0);
        assert_eq!(LoggerState::Start(0), state);

        current_time_us = current_time_us.wrapping_add(1000); // use wrapping_add to handle when time rolls over at max u32.
        _ = state.update_header(&mut logger, &mut encoder, current_time_us);
        assert_eq!(LoggerState::WriteFileHeader, state);

        current_time_us = current_time_us.wrapping_add(1000); // use wrapping_add to handle when time rolls over at max u32.
        _ = state.update_header(&mut logger, &mut encoder, current_time_us);
        assert_eq!(LoggerState::WriteMainFieldsHeader(FieldHeaderIndex::IName(0)), state);

        /*current_time_us = current_time_us.wrapping_add(1000); // use wrapping_add to handle when time rolls over at max u32.
        _ = state.update(&mut logger, &mut encoder, current_time_us, true);
        assert_eq!(StateMachine::LogSlowFieldsHeader, state);

        current_time_us = current_time_us.wrapping_add(1000); // use wrapping_add to handle when time rolls over at max u32.
        _ = state.update(&mut logger, &mut encoder, current_time_us, true);
        assert_eq!(StateMachine::LogSysinfo(0), state);

        current_time_us = current_time_us.wrapping_add(1000); // use wrapping_add to handle when time rolls over at max u32.
        _ = state.update(&mut logger, &mut encoder, current_time_us, true);
        assert_eq!(StateMachine::Start, state);*/

        loop {
            logger.set_main_data(main_data);
            _ = state.update_header(&mut logger, &mut encoder, current_time_us);
            if state == LoggerState::HeaderWritten {
                if encoder.pos != 0 {
                    let Ok(result) = core::str::from_utf8(&encoder.buffer[..encoder.pos]) else {
                        panic!("from_utf8 failed");
                    };
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
        let mut logger = Logger::new(0);
        logger.init(0, 0, false);

        let mut state = StateMachine::default();
        let mut current_time_us: u64 = 0;
        let gyro_pid_msg = GyroPidMessage::new();
        let setpoint_msg = SetpointMessage::new();
        state.start(start);
        let mut run_count = 0;
        loop {
            logger.load_telemetry(current_time_us, gyro_pid_msg, setpoint_msg);
            _ = state.update(&mut logger, &mut writer, current_time_us);
            //let state_i:u32 = state.into();
            //println!("state={state_i}");
            if state == StateMachine::Running {
                if encoder.pos != 0 {
                    let Ok(result) = core::str::from_utf8(&encoder.buffer[..encoder.pos]) else {
                        panic!("from_utf8 failed");
                    };
                    println!("RR__{result}__RR");
                    run_count += 1;
                }
                break;
                if run_count > 10 {
                    break;
                }
            }
            current_time_us = current_time_us.wrapping_add(1000); // use wrapping_add to handle when time rolls over at max u32.
        }
    }*/
}
