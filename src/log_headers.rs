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
    #[allow(clippy::too_many_lines)]
    pub fn log_sys_info(&mut self, writer: &mut SliceWriter, index: usize) -> usize {
        match index {
            0 => {
                writer.write_h_str("Firmware type:Cleanflight\n");
            }
            1 => {
                writer.write_h_str("Firmware revision:Betaflight 4.2.11\n");
                writer.write_h_str("Firmware date:Mar 9 2021 00:00:00\n");
            }
            2 => {
                writer.write_h_str("Board information:\n");
                writer.write_h_str("Log start datetime:0000-01-01T00:00:00.000+00:00\n");
                writer.write_h_str("Craft name:\n");
            }
            3 => {
                writer.write_h_str_u32_ascii("I interval:", self.i_interval);
                writer.write_h_str_u32_ascii("P interval:1/", self.p_interval);
            }
            4 => {
                writer.write_h_str_u32_ascii("minthrottle:", 1070);
                writer.write_h_str_u32_ascii("maxthrottle:", 2000);
            }
            5 => {
                writer.write_h_str("gyro_scale:0x3f800000\n");
                writer.write_h_str("motorOutput:158,2047\n");
                writer.write_h_str_u32_ascii("acc_1G:", 4096);
            }
            6 => {
                writer.write_h_str_u32_ascii("vbatscale:", 110);
                writer.write_h_str("vbatcellvoltage:330,350,430\n");
                writer.write_h_str_u32_ascii("vbatref:", 2466);
                writer.write_h_str("currentSensor:0,250\n");
            }
            7 => {
                writer.write_h_str_u32_ascii("looptime:", self.looptime);
                writer.write_h_str_u32_ascii("gyro_sync_denom:", 1);
                writer.write_h_str_u32_ascii("pid_process_denom:", 1);
            }
            8 => {
                // "P denom" ignored by blackbox-log-view
                writer.write_h_str("P denom:32\n");
            }
            9 => {
                writer.write_h_str("thr_mid:50\n");
                writer.write_h_str("thr_expo:0\n");
                writer.write_h_str("tpa_rate:65\n");
                writer.write_h_str("tpa_breakpoint:1350\n");
                writer.write_h_str("rc_rates:70,70,70\n");
                writer.write_h_str("rc_expo:0,0,0\n");
                writer.write_h_str("rates:75,75,75\n");
                writer.write_h_str("rate_limits:1998,1998,1998\n");
            }
            10 => {
                writer.write_h_str("rollPID:50,102,36\n");
                writer.write_h_str("pitchPID:55,108,38\n");
                writer.write_h_str("yawPID:54,108,0\n");
                writer.write_h_str("levelPID:50,50,75\n");
                writer.write_h_str("magPID:40\n");
                writer.write_h_str("velPID:55,55,75\n");
            }
            11 => {
                writer.write_h_str("dterm_filter_type:0\n");
                writer.write_h_str("dterm_lpf_hz:100\n");
                writer.write_h_str("yaw_lpf_hz:0\n");
                writer.write_h_str("dterm_notch_hz:0\n");
                writer.write_h_str("dterm_notch_cutoff:160\n");
                writer.write_h_str("iterm_windup:50\n");
                writer.write_h_str("vbat_pid_gain:0\n");
                writer.write_h_str("pidAtMinThrottle:1\n");
            }
            12 => {
                writer.write_h_str("anti_gravity_threshold:350\n");
                writer.write_h_str("anti_gravity_gain:1000\n");
                writer.write_h_str("setpoint_relaxation_ratio:50\n");
                writer.write_h_str("dterm_setpoint_weight:100\n");
                writer.write_h_str("acc_limit_yaw:100\n");
                writer.write_h_str("acc_limit:0\n");
                writer.write_h_str("pidsum_limit:500\n");
                writer.write_h_str("pidsum_limit_yaw:400\n");
            }
            13 => {
                writer.write_h_str("deadband:0\n");
                writer.write_h_str("yaw_deadband:0\n");
            }
            14 => {
                writer.write_h_str("gyro_lpf:0\n");
                writer.write_h_str("gyro_lowpass_type:0\n");
                writer.write_h_str("gyro_lowpass_hz:90\n");
                writer.write_h_str("gyro_notch_hz:0,0\n");
                writer.write_h_str("gyro_notch_cutoff:300,100\n");
                writer.write_h_str("acc_lpf_hz:1000\n");
            }
            15 => {
                writer.write_h_str("acc_hardware:1\n");
                writer.write_h_str("baro_hardware:1\n");
                writer.write_h_str("mag_hardware:1\n");
            }
            16 => {
                writer.write_h_str("gyro_cal_on_first_arm:0\n");
                writer.write_h_str("rc_interpolation:2\n");
                writer.write_h_str("rc_interpolation_interval:19\n");
                writer.write_h_str("airmode_activate_throttle:32\n");
            }
            17 => {
                writer.write_h_str("serialrx_provider:3\n");
                writer.write_h_str("use_unsynced_pwm:0\n");
                writer.write_h_str("motor_pwm_protocol:6\n");
                writer.write_h_str("motor_pwm_rate:480\n");
                writer.write_h_str("dshot_idle_value:550\n");
            }
            18 => {
                writer.write_h_str_u32_ascii("debug_mode:", 0);
                writer.write_h_str("features:541130760\n");
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
    use crate::{BlackboxStartParameters, GyroPidMessage};

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
        ctx.init(3);

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
        ctx.init(3);

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
        ctx.init(3);

        let start = BlackboxStartParameters::new();
        let mut state = StateMachine::default();
        let mut current_time_us: u32 = 0;
        let telemetry = GyroPidMessage::new();
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
        ctx.init(3);

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
