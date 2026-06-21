use crate::encoding::write_field_line;
#[cfg(feature = "gps")]
use crate::field_definitions::ConditionalFieldDefinition;
use crate::field_definitions::{MainFieldDefinition, SimpleFieldDefinition};
use crate::logger::Logger;
use crate::{BlackboxWriter, SliceEncoder};

// order of headers in log file is:
// 1. file header
// 2. main fields header
// 3. slow fields header
// 4. system info
// After the headers we have the logging data.
impl Logger {
    pub fn log_file_header(encoder: &mut SliceEncoder) -> usize {
        encoder.write_h_str("Product:Blackbox flight data recorder by Nicholas Sherlock\n");
        encoder.write_h_str("Data version:2\n");
        encoder.pos
    }

    // Note: this mini state machine will go once I start using async and await.
    pub fn log_main_fields_header(&mut self, encoder: &mut SliceEncoder, index: usize) -> usize {
        const MAIN_FIELDS: &[MainFieldDefinition] = crate::field_arrays::BLACKBOX_MAIN_FIELDS;

        let filter = |f: &MainFieldDefinition| self.conditions.test(f.condition);

        match index {
            0 => {
                // Name line. Note: This can exceed 500 bytes.
                // Currently using buffer of size 1024, this will be solved once I start using async and await.
                write_field_line(encoder, 'I', "name", MAIN_FIELDS.iter().filter(|&f| filter(f)), |w, f| {
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
                write_field_line(encoder, 'I', "signed", filtered, |w, f| {
                    w.write_u8_ascii(f.is_signed);
                });
            }
            2 => {
                // I Predictor line
                let filtered = MAIN_FIELDS.iter().filter(|&f| filter(f));
                write_field_line(encoder, 'I', "predictor", filtered, |w, f| {
                    w.write_u8_ascii(f.i_predict);
                });
            }
            3 => {
                // I Encoding line
                let filtered = MAIN_FIELDS.iter().filter(|&f| filter(f));
                write_field_line(encoder, 'I', "encoding", filtered, |w, f| {
                    w.write_u8_ascii(f.i_encode);
                });
            }
            4 => {
                // P Predictor line
                let filtered = MAIN_FIELDS.iter().filter(|&f| filter(f));
                write_field_line(encoder, 'P', "predictor", filtered, |w, f| {
                    w.write_u8_ascii(f.p_predict);
                });
            }
            5 => {
                // P Encoding line
                let filtered = MAIN_FIELDS.iter().filter(|&f| filter(f));
                write_field_line(encoder, 'P', "encoding", filtered, |w, f| {
                    w.write_u8_ascii(f.p_encode);
                });
            }
            _ => {
                return 0;
            }
        }

        encoder.pos
    }

    #[allow(clippy::unused_self)]
    pub fn log_slow_fields_header(&mut self, encoder: &mut SliceEncoder) -> usize {
        const SLOW_FIELDS: &[SimpleFieldDefinition; SimpleFieldDefinition::SLOW_FIELD_COUNT] =
            &crate::field_arrays::BLACKBOX_SLOW_FIELDS;
        let filter = |_: &SimpleFieldDefinition| true;

        // Name line.
        write_field_line(encoder, 'S', "name", SLOW_FIELDS.iter().filter(|&f| filter(f)), |w, f| {
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
        write_field_line(encoder, 'S', "signed", filtered, |w, f| {
            w.write_u8_ascii(f.is_signed);
        });
        // Predictor line
        let filtered = SLOW_FIELDS.iter().filter(|&f| filter(f));
        write_field_line(encoder, 'S', "predictor", filtered, |w, f| {
            w.write_u8_ascii(f.predict);
        });
        // Encoding line
        let filtered = SLOW_FIELDS.iter().filter(|&f| filter(f));
        write_field_line(encoder, 'S', "encoding", filtered, |w, f| {
            w.write_u8_ascii(f.encode);
        });

        encoder.pos
    }

    #[cfg(feature = "gps")]
    pub fn log_gps_h_fields_header(&mut self, encoder: &mut SliceEncoder) -> usize {
        const GPS_H_FIELDS: &[SimpleFieldDefinition; SimpleFieldDefinition::GPS_H_FIELD_COUNT] =
            &crate::field_arrays::BLACKBOX_GPS_H_FIELDS;
        let filter = |_: &SimpleFieldDefinition| true;

        // Name line.
        write_field_line(encoder, 'H', "name", GPS_H_FIELDS.iter().filter(|&f| filter(f)), |w, f| {
            w.write_str(f.name);
            let index = f.field_name_index;
            if index >= 0 {
                w.write_char('[');
                w.write_u8_ascii(index.cast_unsigned());
                w.write_char(']');
            }
        });
        // Signed line
        let filtered = GPS_H_FIELDS.iter().filter(|&f| filter(f));
        write_field_line(encoder, 'H', "signed", filtered, |w, f| {
            w.write_u8_ascii(f.is_signed);
        });
        // Predictor line
        let filtered = GPS_H_FIELDS.iter().filter(|&f| filter(f));
        write_field_line(encoder, 'H', "predictor", filtered, |w, f| {
            w.write_u8_ascii(f.predict);
        });
        // Encoding line
        let filtered = GPS_H_FIELDS.iter().filter(|&f| filter(f));
        write_field_line(encoder, 'H', "encoding", filtered, |w, f| {
            w.write_u8_ascii(f.encode);
        });

        encoder.pos
    }

    #[cfg(feature = "gps")]
    pub fn log_gps_g_fields_header(&mut self, encoder: &mut SliceEncoder) -> usize {
        const GPS_G_FIELDS: &[ConditionalFieldDefinition; ConditionalFieldDefinition::GPS_G_FIELD_COUNT] =
            &crate::field_arrays::BLACKBOX_GPS_G_FIELDS;
        let filter = |_: &ConditionalFieldDefinition| true;

        // Name line.
        write_field_line(encoder, 'G', "name", GPS_G_FIELDS.iter().filter(|&f| filter(f)), |w, f| {
            w.write_str(f.name);
            let index = f.field_name_index;
            if index >= 0 {
                w.write_char('[');
                w.write_u8_ascii(index.cast_unsigned());
                w.write_char(']');
            }
        });
        // Signed line
        let filtered = GPS_G_FIELDS.iter().filter(|&f| filter(f));
        write_field_line(encoder, 'G', "signed", filtered, |w, f| {
            w.write_u8_ascii(f.is_signed);
        });
        // Predictor line
        let filtered = GPS_G_FIELDS.iter().filter(|&f| filter(f));
        write_field_line(encoder, 'G', "predictor", filtered, |w, f| {
            w.write_u8_ascii(f.predict);
        });
        // Encoding line
        let filtered = GPS_G_FIELDS.iter().filter(|&f| filter(f));
        write_field_line(encoder, 'G', "encoding", filtered, |w, f| {
            w.write_u8_ascii(f.encode);
        });

        encoder.pos
    }

    // Note: this mini state machine will go once I start using async and await.
    #[allow(clippy::too_many_lines)]
    pub fn log_sys_info(&mut self, encoder: &mut SliceEncoder, index: usize) -> usize {
        match index {
            0 => {
                encoder.write_h_str("Firmware type:Cleanflight\n");
            }
            1 => {
                encoder.write_h_str("Firmware revision:Betaflight 4.2.11\n");
                //encoder.write_h_str("Firmware date:Mar 9 2021 00:00:00\n");
            }
            2 => {
                //encoder.write_h_str("Board information:\n");
                encoder.write_h_str("Log start datetime:0000-01-01T00:00:00.000+00:00\n");
                //encoder.write_h_str("Craft name:\n");
            }
            3 => {
                encoder.write_h_str_u32_ascii("I interval:", self.i_interval);
                encoder.write_h_str_u32_ascii("P interval:1/", self.p_interval);
            }
            4 => {
                encoder.write_h_str_u32_ascii("looptime:", self.looptime);
                encoder.write_h_str_u32_ascii("gyro_sync_denom:", 1);
                encoder.write_h_str_u32_ascii("pid_process_denom:", 1);
            }
            5 => {
                // "P denom" ignored by blackbox-log-view
                encoder.write_h_str("P denom:32\n");
                encoder.write_h_str_u32_ascii("debug_mode:", 0);
                encoder.write_h_str("features:541130760\n");
            }
            6 => {
                encoder.write_h_str("gyro_scale:0x3f800000\n");
                encoder.write_h_str("motorOutput:");
                encoder.write_u32_ascii(u32::from(self.motor_output_min));
                encoder.write_byte(b',');
                encoder.write_u32_ascii(u32::from(self.motor_output_max));
                encoder.write_char('\n');

                encoder.write_h_str_u32_ascii("acc_1G:", 4096);
            }
            7 => {
                encoder.write_h_str_u16_ascii("minthrottle:", self.min_throttle);
                encoder.write_h_str_u16_ascii("maxthrottle:", self.max_throttle);
            }
            8 => {
                encoder.write_h_str_u32_ascii("vbatscale:", 110);
                encoder.write_h_str("vbatcellvoltage:330,350,430\n");
                encoder.write_h_str_u16_ascii("vbatref:", self.vbat_reference);
                encoder.write_h_str("currentSensor:0,250\n");
            }
            /*9 => {
                encoder.write_h_str("thr_mid:50\n");
                encoder.write_h_str("thr_expo:0\n");
                encoder.write_h_str("tpa_rate:65\n");
                encoder.write_h_str("tpa_breakpoint:1350\n");
                encoder.write_h_str("rc_rates:70,70,70\n");
                encoder.write_h_str("rc_expo:0,0,0\n");
                encoder.write_h_str("rates:75,75,75\n");
                encoder.write_h_str("rate_limits:1998,1998,1998\n");
            }
            10 => {
                encoder.write_h_str("rollPID:50,102,36\n");
                encoder.write_h_str("pitchPID:55,108,38\n");
                encoder.write_h_str("yawPID:54,108,0\n");
                encoder.write_h_str("levelPID:50,50,75\n");
                encoder.write_h_str("magPID:40\n");
                encoder.write_h_str("velPID:55,55,75\n");
            }
            11 => {
                encoder.write_h_str("dterm_filter_type:0\n");
                encoder.write_h_str("dterm_lpf_hz:100\n");
                encoder.write_h_str("yaw_lpf_hz:0\n");
                encoder.write_h_str("dterm_notch_hz:0\n");
                encoder.write_h_str("dterm_notch_cutoff:160\n");
                encoder.write_h_str("iterm_windup:50\n");
                encoder.write_h_str("vbat_pid_gain:0\n");
                encoder.write_h_str("pidAtMinThrottle:1\n");
            }
            12 => {
                encoder.write_h_str("anti_gravity_threshold:350\n");
                encoder.write_h_str("anti_gravity_gain:1000\n");
                encoder.write_h_str("setpoint_relaxation_ratio:50\n");
                encoder.write_h_str("dterm_setpoint_weight:100\n");
                encoder.write_h_str("acc_limit_yaw:100\n");
                encoder.write_h_str("acc_limit:0\n");
                encoder.write_h_str("pidsum_limit:500\n");
                encoder.write_h_str("pidsum_limit_yaw:400\n");
            }
            13 => {
                encoder.write_h_str("deadband:0\n");
                encoder.write_h_str("yaw_deadband:0\n");
            }
            14 => {
                encoder.write_h_str("gyro_lpf:0\n");
                encoder.write_h_str("gyro_lowpass_type:0\n");
                encoder.write_h_str("gyro_lowpass_hz:90\n");
                encoder.write_h_str("gyro_notch_hz:0,0\n");
                encoder.write_h_str("gyro_notch_cutoff:300,100\n");
                encoder.write_h_str("acc_lpf_hz:1000\n");
            }
            15 => {
                encoder.write_h_str("acc_hardware:1\n");
                encoder.write_h_str("baro_hardware:1\n");
                encoder.write_h_str("mag_hardware:1\n");
            }
            16 => {
                encoder.write_h_str("gyro_cal_on_first_arm:0\n");
                encoder.write_h_str("rc_interpolation:2\n");
                encoder.write_h_str("rc_interpolation_interval:19\n");
                encoder.write_h_str("airmode_activate_throttle:32\n");
            }
            17 => {
                encoder.write_h_str("serialrx_provider:3\n");
                encoder.write_h_str("use_unsynced_pwm:0\n");
                encoder.write_h_str("motor_pwm_protocol:6\n");
                encoder.write_h_str("motor_pwm_rate:480\n");
                encoder.write_h_str("dshot_idle_value:550\n");
            }*/
            _ => {
                return 0;
            }
        }

        encoder.pos
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use crate::state_machine::StateMachine;
    use crate::{BlackboxStartParameters, GyroPidMessage, SetpointMessage};

    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<Logger>();
    }
    #[test]
    fn test_new() {
        let ctx = Logger::new(0);
        assert!(!ctx.logged_any_frames);
    }

    #[test]
    fn log_file_header() {
        let mut buffer = [0u8; 2048];
        //let mut sd_card = MockSdCard::new("state_machine_log.bbl");
        let pos = {
            let mut encoder = SliceEncoder { buffer: &mut buffer, pos: 0 };

            _ = Logger::log_file_header(&mut encoder);

            // Convert the written portion to a string for validation
            let result = core::str::from_utf8(&encoder.buffer[..encoder.pos]).unwrap();
            // Print for manual inspection (if running with `cargo test -- --nocapture`)
            println!("\nHEADER\n{result}\n");
            assert!(result.contains("H Product:Blackbox"));
            encoder.pos
        };
        let result = core::str::from_utf8(&buffer[..pos]).unwrap();
        println!("\nBUFFER\n{result}\n");

        //sd_card.write_all(&buffer[..pos]);
    }
    #[test]
    fn log_main_fields_header() {
        let mut buffer = [0u8; 2048];
        let mut encoder = SliceEncoder { buffer: &mut buffer, pos: 0 };
        let mut ctx = Logger::new(0);
        ctx.init(0, 0);

        let mut index: usize = 0;
        loop {
            if ctx.log_main_fields_header(&mut encoder, index) == 0 {
                break;
            }
            index += 1;
        }

        // Convert the written portion to a string for validation
        #[allow(clippy::unwrap_used)]
        let result = core::str::from_utf8(&encoder.buffer[..encoder.pos]).unwrap();
        // Print for manual inspection (if running with `cargo test -- --nocapture`)
        println!("\nMAIN FIELDS HEADER\n{result}\n");
    }
    #[test]
    fn log_slow_fields_header() {
        let mut buffer = [0u8; 2048];
        let mut encoder = SliceEncoder { buffer: &mut buffer, pos: 0 };
        let mut ctx = Logger::new(0);
        ctx.init(0, 0);

        let len = ctx.log_slow_fields_header(&mut encoder);
        assert_eq!(encoder.pos, len);

        // Convert the written portion to a string for validation
        #[allow(clippy::unwrap_used)]
        let result = core::str::from_utf8(&encoder.buffer[..encoder.pos]).unwrap();
        // Print for manual inspection (if running with `cargo test -- --nocapture`)
        println!("\nSLOW FIELDS HEADER\n{result}\n");
    }
    #[test]
    fn log_sys_info() {
        let mut buffer = [0u8; 2048];
        let mut encoder = SliceEncoder { buffer: &mut buffer, pos: 0 };
        let mut ctx = Logger::new(0);

        let mut index: usize = 0;
        loop {
            if ctx.log_sys_info(&mut encoder, index) == 0 {
                break;
            }
            index += 1;
        }

        // Convert the written portion to a string for validation
        #[allow(clippy::unwrap_used)]
        let result = core::str::from_utf8(&encoder.buffer[..encoder.pos]).unwrap();
        // Print for manual inspection (if running with `cargo test -- --nocapture`)
        println!("\nSYS INFO\n{result}\n");
    }
    #[test]
    fn state_machine_headers() {
        let mut buffer = [0u8; 4096];
        let mut encoder = SliceEncoder { buffer: &mut buffer, pos: 0 };
        let mut ctx = Logger::new(0);
        //let mut _sd_card = MockSdCard::new("state_machine_log.bbl");
        ctx.init(0, 0);

        let start = BlackboxStartParameters::new();
        let mut state = StateMachine::default();
        let mut current_time_us: u32 = 0;
        let gyro_pid_msg = GyroPidMessage::new();
        let setpoint_msg = SetpointMessage::new();
        println!("\nSTATE MACHINE HEADERS\n");
        state.start(start);
        loop {
            ctx.load_telemetry(current_time_us, gyro_pid_msg, setpoint_msg);
            _ = state.update(&mut ctx, &mut encoder, current_time_us, true);
            if state == StateMachine::Running {
                if encoder.pos != 0 {
                    #[allow(clippy::unwrap_used)]
                    let result = core::str::from_utf8(&encoder.buffer[..encoder.pos]).unwrap();
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
        let mut ctx = Logger::new(0);
        ctx.init(0, 0);

        let start = BlackboxStartParameters::new();
        let mut state = StateMachine::default();
        let mut current_time_us: u32 = 0;
        let gyro_pid_msg = GyroPidMessage::new();
        let setpoint_msg = SetpointMessage::new();
        state.start(start);
        let mut run_count = 0;
        loop {
            ctx.load_telemetry(current_time_us, gyro_pid_msg, setpoint_msg);
            _ = state.update(&mut ctx, &mut writer, current_time_us);
            //let state_i:u32 = state.into();
            //println!("state={state_i}");
            if state == StateMachine::Running {
                if encoder.pos != 0 {
                    #[allow(clippy::unwrap_used)]
                    let result = core::str::from_utf8(&encoder.buffer[..encoder.pos]).unwrap();
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
