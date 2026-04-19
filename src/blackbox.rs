use crate::{FieldCondition, LogFieldSelect, SliceWriter};
use crate::{GpsState, MainState, SlowState};
use receivers::BitSet64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blackbox {
    iteration: u32,
    motor_count: usize,
    servo_count: usize,
    debug_mode: u32,
    motor_output_low: i16,
    i_frame_index: u32,
    i_interval: u32,
    p_frame_index: u32,
    p_interval: u32,
    s_frame_index: u32,
    s_interval: u32,
    logged_any_frames: bool,
    condition_cache: BitSet64,
    log_select_enabled: u32,
    vbat_reference: i16,
    slow_state: SlowState,
    gps_state: GpsState,
    main_states: [MainState; 3],
    buf: [u8; 1024],
}

impl Default for Blackbox {
    fn default() -> Self {
        Self::new()
    }
}

impl Blackbox {
    pub fn new() -> Self {
        Self {
            iteration: 0,
            motor_count: 4,
            servo_count: 0,
            debug_mode: 0,
            motor_output_low: 750,
            i_frame_index: 0,
            i_interval: 0,
            p_frame_index: 0,
            p_interval: 0,
            s_frame_index: 0,
            s_interval: 0,
            logged_any_frames: false,
            condition_cache: BitSet64::default(),
            log_select_enabled: 0,
            vbat_reference: 0,
            slow_state: SlowState::default(),
            gps_state: GpsState::default(),
            main_states: <[MainState; 3]>::default(),
            buf: [0u8; 1024],
        }
    }
}
/// Build condition cache, called from start().
impl Blackbox {
    pub fn build_field_condition_cache(&mut self) {
        self.condition_cache.reset_all();
        for condition in FieldCondition::FIRST..FieldCondition::LAST {
            if self.test_field_condition_uncached(condition) {
                _ = self.condition_cache.set(condition);
            }
        }
    }
}

impl Blackbox {
    //fn field_enabled(enabled_mask:u32, field:LogFieldSelect) -> bool { enabled_mask & (field as u32) }
    //pub fn is_field_enabled(&self, field:LogFieldSelect) ->bool { field_enabled(self.log_select_enabled, field) }
    // Helper function to check if a field is enabled
    fn field_enabled(enabled_mask: u32, field: u32) -> bool {
        enabled_mask & field != 0
    }

    // Public method to check if a log field is enabled
    pub fn is_field_enabled(&self, field: u32) -> bool {
        Self::field_enabled(self.log_select_enabled, field)
    }

    //Called from build_field_condition_cache(), which is called from start()
    // Test condition without caching
    pub fn test_field_condition_uncached(&self, condition: u8) -> bool {
        match condition {
            FieldCondition::ALWAYS => true,

            FieldCondition::AT_LEAST_MOTORS_1
            | FieldCondition::AT_LEAST_MOTORS_2
            | FieldCondition::AT_LEAST_MOTORS_3
            | FieldCondition::AT_LEAST_MOTORS_4
            | FieldCondition::AT_LEAST_MOTORS_5
            | FieldCondition::AT_LEAST_MOTORS_6
            | FieldCondition::AT_LEAST_MOTORS_7
            | FieldCondition::AT_LEAST_MOTORS_8 => {
                self.is_field_enabled(LogFieldSelect::MOTOR)
                    && self.motor_count > (condition - FieldCondition::AT_LEAST_MOTORS_1) as usize
            }

            FieldCondition::MOTOR_1_HAS_RPM
            | FieldCondition::MOTOR_2_HAS_RPM
            | FieldCondition::MOTOR_3_HAS_RPM
            | FieldCondition::MOTOR_4_HAS_RPM
            | FieldCondition::MOTOR_5_HAS_RPM
            | FieldCondition::MOTOR_6_HAS_RPM
            | FieldCondition::MOTOR_7_HAS_RPM
            | FieldCondition::MOTOR_8_HAS_RPM => {
                self.is_field_enabled(LogFieldSelect::MOTOR_RPM)
                    && self.motor_count > (condition - FieldCondition::MOTOR_1_HAS_RPM) as usize
            }

            FieldCondition::SERVOS => self.is_field_enabled(LogFieldSelect::SERVO) && self.servo_count > 0,

            FieldCondition::PID => self.is_field_enabled(LogFieldSelect::PID),

            FieldCondition::PID_K => {
                self.is_field_enabled(LogFieldSelect::PID) && self.is_field_enabled(LogFieldSelect::PID_KTERM)
            }
            FieldCondition::PID_D_ROLL => {
                self.is_field_enabled(LogFieldSelect::PID) && self.is_field_enabled(LogFieldSelect::PID_DTERM_ROLL)
            }
            FieldCondition::PID_D_PITCH => {
                self.is_field_enabled(LogFieldSelect::PID) && self.is_field_enabled(LogFieldSelect::PID_DTERM_PITCH)
            }
            FieldCondition::PID_D_YAW => {
                self.is_field_enabled(LogFieldSelect::PID) && self.is_field_enabled(LogFieldSelect::PID_DTERM_YAW)
            }
            FieldCondition::PID_S_ROLL => {
                self.is_field_enabled(LogFieldSelect::PID) && self.is_field_enabled(LogFieldSelect::PID_STERM_ROLL)
            }
            FieldCondition::PID_S_PITCH => {
                self.is_field_enabled(LogFieldSelect::PID) && self.is_field_enabled(LogFieldSelect::PID_STERM_PITCH)
            }
            FieldCondition::PID_S_YAW => {
                self.is_field_enabled(LogFieldSelect::PID) && self.is_field_enabled(LogFieldSelect::PID_STERM_YAW)
            }

            FieldCondition::RC_COMMANDS => self.is_field_enabled(LogFieldSelect::RC_COMMANDS),
            FieldCondition::SETPOINT => self.is_field_enabled(LogFieldSelect::SETPOINT),
            FieldCondition::MAGNETOMETER => self.is_field_enabled(LogFieldSelect::MAGNETOMETER),
            FieldCondition::BAROMETER => self.is_field_enabled(LogFieldSelect::BAROMETER),
            FieldCondition::BATTERY_VOLTAGE => self.is_field_enabled(LogFieldSelect::BATTERY_VOLTAGE),
            FieldCondition::BATTERY_CURRENT => self.is_field_enabled(LogFieldSelect::BATTERY_CURRENT),
            FieldCondition::RANGEFINDER => self.is_field_enabled(LogFieldSelect::RANGEFINDER),
            FieldCondition::RSSI => self.is_field_enabled(LogFieldSelect::RSSI),

            FieldCondition::NOT_LOGGING_EVERY_FRAME => self.p_interval != self.i_interval,

            FieldCondition::GYRO => self.is_field_enabled(LogFieldSelect::GYRO),
            FieldCondition::GYRO_UNFILTERED => self.is_field_enabled(LogFieldSelect::GYRO_UNFILTERED),
            FieldCondition::ACC => self.is_field_enabled(LogFieldSelect::ACCELEROMETER),
            FieldCondition::ATTITUDE => self.is_field_enabled(LogFieldSelect::ATTITUDE),

            FieldCondition::DEBUG => self.is_field_enabled(LogFieldSelect::DEBUG) && self.debug_mode != 0,

            // Handle any unknown condition
            _ => false,
        }
    }
}

/// Write the contents of slow_state to the log as an S frame.
impl Blackbox {
    pub fn log_s_frame(&mut self) {
        self.s_frame_index = 0;

        let mut encoder = SliceWriter { buffer: &mut self.buf, pos: 0 };
        encoder.begin_frame(b'S');

        encoder.write_unsigned_vb(self.slow_state.flight_mode_flags);
        encoder.write_unsigned_vb(u32::from(self.slow_state.state_flags));

        // Most of the time these three values will be able to pack into one byte.
        let values = [
            i32::from(self.slow_state.failsafe_phase),
            i32::from(self.slow_state.rx_signal_received),
            i32::from(self.slow_state.rx_flight_channel_is_valid),
        ];

        encoder.write_tag2_3s32(values);

        encoder.end_frame();
    }

    pub fn log_s_frame2(&mut self, encoder: &mut SliceWriter) {
        self.s_frame_index = 0;

        encoder.begin_frame(b'S');

        encoder.write_unsigned_vb(self.slow_state.flight_mode_flags);
        encoder.write_unsigned_vb(u32::from(self.slow_state.state_flags));

        // Most of the time these three values will be able to pack into one byte.
        let values = [
            i32::from(self.slow_state.failsafe_phase),
            i32::from(self.slow_state.rx_signal_received),
            i32::from(self.slow_state.rx_flight_channel_is_valid),
        ];

        encoder.write_tag2_3s32(values);

        encoder.end_frame();
    }
}

impl Blackbox {
    pub fn log_h_frame(&mut self) {
        let mut encoder = SliceWriter { buffer: &mut self.buf, pos: 0 };
        encoder.begin_frame(b'H');

        encoder.write_signed_vb(self.gps_state.home_latitude_degrees_1e7);
        encoder.write_signed_vb(self.gps_state.home_longitude_degrees_1e7);

        encoder.write_signed_vb(self.gps_state.home_altitude_cm / 10);

        encoder.end_frame();
    }
}

impl Blackbox {
    pub fn log_i_frame(&mut self) {
        {
            let mut encoder = SliceWriter { buffer: &mut self.buf, pos: 0 };
            encoder.begin_frame(b'I');

            encoder.write_unsigned_vb(self.iteration);

            let main_state = self.main_states[0];

            encoder.write_unsigned_vb(main_state.time_us);

            if self.condition_cache.test(FieldCondition::PID) {
                encoder.write_signed_16_vb_array(&main_state.axis_pid_p);
                encoder.write_signed_16_vb_array(&main_state.axis_pid_i);

                if self.condition_cache.test(FieldCondition::PID_D_ROLL) {
                    encoder.write_s16(main_state.axis_pid_d[0]);
                }
                if self.condition_cache.test(FieldCondition::PID_D_PITCH) {
                    encoder.write_s16(main_state.axis_pid_d[1]);
                }
                if self.condition_cache.test(FieldCondition::PID_D_YAW) {
                    encoder.write_s16(main_state.axis_pid_d[2]);
                }

                if self.condition_cache.test(FieldCondition::PID_K) {
                    encoder.write_signed_16_vb_array(&main_state.axis_pid_k);
                }

                if self.condition_cache.test(FieldCondition::PID_S_ROLL) {
                    encoder.write_s16(main_state.axis_pid_s[0]);
                }
                if self.condition_cache.test(FieldCondition::PID_S_PITCH) {
                    encoder.write_s16(main_state.axis_pid_s[1]);
                }
                if self.condition_cache.test(FieldCondition::PID_S_YAW) {
                    encoder.write_s16(main_state.axis_pid_s[2]);
                }

                if self.condition_cache.test(FieldCondition::RC_COMMANDS) {
                    // Write roll, pitch and yaw first, these are signed values in the range [-500,500]
                    encoder.write_signed_16_vb_array(&main_state.rc_commands);

                    // Write the throttle separately from the rest of the RC data as it's unsigned.
                    // Throttle lies in range [PWM_RANGE_MIN,PWM_RANGE_MAX], ie [1000,2000]
                    encoder.write_s16(main_state.rc_commands[MainState::THROTTLE]);
                }

                if self.condition_cache.test(FieldCondition::SETPOINT) {
                    // Write setpoint roll, pitch, yaw, and throttle
                    encoder.write_signed_16_vb_array(&main_state.setpoints);
                }

                if self.condition_cache.test(FieldCondition::BATTERY_VOLTAGE) {
                    //Our voltage is expected to decrease over the course of the flight, so store our difference from
                    //the reference:
                    // Write 14 bits even if the number is negative (which would otherwise result in 32 bits)
                    encoder.write_s16((self.vbat_reference - main_state.vbat_latest) & 0x3FFF);
                }

                if self.condition_cache.test(FieldCondition::BATTERY_CURRENT) {
                    // 12bit value directly from ADC
                    encoder.write_signed_vb(main_state.amperage_latest);
                }

                #[cfg(feature = "magnetometer")]
                if self.condition_cache.test(FieldCondition::MAGNETOMETER) {
                    encoder.write_signed_16_vb_array(&main_state.mag_adc[0], XYZ_AXIS_COUNT);
                }

                #[cfg(feature = "barometer")]
                if self.condition_cache.test(FieldCondition::BAROMETER) {
                    encoder.write_signed_vb(main_state.baro_altitude);
                }

                #[cfg(feature = "rangefinder")]
                if self.condition_cache.test(FieldCondition::RANGEFINDER) {
                    encoder.write_signed_vb(main_state.surface_raw);
                }

                if self.condition_cache.test(FieldCondition::RSSI) {
                    encoder.write_s16(main_state.rssi);
                }

                if self.condition_cache.test(FieldCondition::GYRO) {
                    encoder.write_signed_16_vb_array(&main_state.gyro_adc);
                }

                if self.condition_cache.test(FieldCondition::GYRO_UNFILTERED) {
                    encoder.write_signed_16_vb_array(&main_state.gyro_unfiltered);
                }

                if self.condition_cache.test(FieldCondition::ACC) {
                    encoder.write_signed_16_vb_array(&main_state.acc_adc);
                }

                if self.condition_cache.test(FieldCondition::ATTITUDE) {
                    encoder.write_signed_16_vb_array(&main_state.orientation);
                }

                if self.condition_cache.test(FieldCondition::DEBUG) {
                    encoder.write_signed_16_vb_array(&main_state.debug);
                }

                if Self::field_enabled(self.log_select_enabled, LogFieldSelect::MOTOR) {
                    //Motors can be below minimum output when disarmed, but that doesn't happen much
                    encoder.write_s16(main_state.motors[0] - self.motor_output_low);

                    //Motors tend to be similar to each other so use the first motor's value as a predictor of the others
                    for ii in 1..self.motor_count {
                        encoder.write_s16(main_state.motors[ii] - main_state.motors[0]);
                    }
                }
                #[cfg(feature = "servos")]
                if self.condition_cache.test(FieldCondition::SERVOS) {
                    let out: [i32; MainState::MAX_SUPPORTED_SERVO_COUNT] =
                        std::array::from_fn(|i| i32::from(main_state.servos[i]) - 1500);
                    encoder.write_tag8_8svb(&out);
                }
                #[cfg(feature = "dshot_telemetry")]
                if Self::field_enabled(self.log_select_enabled, LogFieldSelect::MOTOR) {
                    for erpm in main_state.erpms {
                        encoder.write_s16(erpm);
                    }
                }
            }

            encoder.end_frame();
        }
        // 2=1
        // 1=0
        // 0=2
        let history_to_save = self.main_states[2];

        // The current state becomes the new "before" state
        self.main_states[1] = self.main_states[0];
        // And since we have no other history, we also use it for the "before, before" state
        self.main_states[2] = self.main_states[0];
        // And advance the current state over to a blank space ready to be filled
        // _main_state_history[0] = ((_main_state_history[0] - &_main_state_history_ring[1]) % 3) + &_main_state_history_ring[0];
        self.main_states[0] = history_to_save;

        self.logged_any_frames = true;
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    #![allow(unused_results)]

    #[allow(unused)]
    use super::*;

    #[allow(unused)]
    fn is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<Blackbox>();
    }
    #[test]
    fn new() {
        let blackbox = Blackbox::new();
        assert_eq!(0, blackbox.iteration);
    }
}
