use crate::SliceWriter;
use crate::field_definitions::{FieldCondition, LogFieldSelect};
use crate::logger::Logger;
use crate::states::MainState;

#[cfg(test)]
use crate::field_definitions::{FieldEncoding, FieldPredictor, MainFieldDefinition};

/// Write the contents of slow_state to the log as an s_frame.
/// Returns the length written.
macro_rules! assert_i_field_encoding {
    ($name:expr, $expected_predict:expr, $expected_encode:expr) => {
        #[cfg(test)]
        {
            let field = MainFieldDefinition::find_by_name($name).expect(concat!("Field not found: ", $name));
            assert_eq!(field.i_predict, $expected_predict, "I PREDICT mismatch for field: \"{}\"", $name);
            assert_eq!(field.i_encode, $expected_encode, "I ENCODE mismatch for field: \"{}\"", $name);
        }
    };
}

macro_rules! assert_p_field_encoding {
    ($name:expr, $expected_predict:expr, $expected_encode:expr) => {
        #[cfg(test)]
        {
            let field = MainFieldDefinition::find_by_name($name).expect(concat!("Field not found: ", $name));
            assert_eq!(field.p_predict, $expected_predict, "P PREDICT mismatch for field: \"{}\"", $name);
            assert_eq!(field.p_encode, $expected_encode, "P ENCODE mismatch for field: \"{}\"", $name);
        }
    };
}

impl Logger {
    pub fn log_s_frame(&mut self, encoder: &mut SliceWriter) -> usize {
        self.logged_any_frames = true;
        self.s_frame_index = 0;
        self.new_slow_state = false;

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

        encoder.end_frame()
    }

    /// GPS home frame: h_frame.
    pub fn log_h_frame(&mut self, encoder: &mut SliceWriter) -> usize {
        self.logged_any_frames = true;

        encoder.begin_frame(b'H');

        encoder.write_signed_vb(self.gps_state.home_latitude_degrees_1e7);
        encoder.write_signed_vb(self.gps_state.home_longitude_degrees_1e7);

        encoder.write_signed_vb(self.gps_state.home_altitude_cm / 10);

        encoder.end_frame()
    }

    /// GPS frame: g_frame.
    pub fn log_g_frame(&mut self, current_time_us: u32, encoder: &mut SliceWriter) -> usize {
        self.logged_any_frames = true;
        self.new_gps_state = false;

        encoder.begin_frame(b'G');

        // If we're logging every frame, then a GPS frame always appears just after a frame with the
        // current_time timestamp in the log, so the reader can just use that timestamp for the GPS frame.
        // If we're not logging every frame, we need to store the time of this GPS frame.
        if self.conditions.test(FieldCondition::NOT_LOGGING_EVERY_FRAME) {
            // Predict the time of the last frame in the main log
            encoder.write_unsigned_vb(current_time_us - self.main_states[self.main_state_index_current].time_us);
        }

        encoder.write_unsigned_vb(u32::from(self.gps_state.satellite_count));
        encoder.write_signed_vb(self.gps_state.latitude_degrees_1e7 - self.home_latitude_degrees_1e7);
        encoder.write_signed_vb(self.gps_state.longitude_degrees_1e7 - self.home_longitude_degrees_1e7);
        // log altitude in increments of 0.1m
        encoder.write_signed_vb(self.gps_state.altitude_cm / 10);

        #[allow(clippy::cast_sign_loss)]
        //if self.config.gps_use_3d_speed {
        //    encoder.write_unsigned_vb(self.gps_state.speed3d_cmps as u32);
        //} else {
        encoder.write_unsigned_vb(self.gps_state.ground_speed_cmps as u32);
        //}

        #[allow(clippy::cast_sign_loss)]
        encoder.write_unsigned_vb(self.gps_state.ground_course_deci_degrees as u32);

        encoder.write_signed_vb_16(self.gps_state.velocity_north_cmps);
        encoder.write_signed_vb_16(self.gps_state.velocity_east_cmps);
        encoder.write_signed_vb_16(self.gps_state.velocity_down_cmps);

        encoder.end_frame()
    }

    #[allow(clippy::too_many_lines)]
    /// Intra frame: i_frame.
    pub fn log_i_frame(&mut self, encoder: &mut SliceWriter) -> usize {
        self.logged_any_frames = true;

        encoder.begin_frame(b'I');

        assert_i_field_encoding!("loopIteration", FieldPredictor::ZERO, FieldEncoding::UNSIGNED_VB);
        encoder.write_unsigned_vb(self.iteration);

        let current = &self.main_states[self.main_state_index_current];

        encoder.write_unsigned_vb(current.time_us);

        if self.conditions.test(FieldCondition::PID) {
            assert_i_field_encoding!("axisP", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            encoder.write_signed_vb_array(&current.pid_p);
            assert_i_field_encoding!("axisI", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            encoder.write_signed_vb_array(&current.pid_i);

            assert_i_field_encoding!("axisD", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            if self.conditions.test(FieldCondition::PID_D_ROLL) {
                encoder.write_signed_vb(current.pid_d[0]);
            }
            if self.conditions.test(FieldCondition::PID_D_PITCH) {
                encoder.write_signed_vb(current.pid_d[1]);
            }
            if self.conditions.test(FieldCondition::PID_D_YAW) {
                encoder.write_signed_vb(current.pid_d[2]);
            }

            assert_i_field_encoding!("axisF", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            if self.conditions.test(FieldCondition::PID_K) {
                encoder.write_signed_vb_array(&current.pid_k);
            }

            assert_i_field_encoding!("axisS", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            if self.conditions.test(FieldCondition::PID_S_ROLL) {
                encoder.write_signed_vb(current.pid_s[0]);
            }
            if self.conditions.test(FieldCondition::PID_S_PITCH) {
                encoder.write_signed_vb(current.pid_s[1]);
            }
            if self.conditions.test(FieldCondition::PID_S_YAW) {
                encoder.write_signed_vb(current.pid_s[2]);
            }

            assert_i_field_encoding!("rc_command", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            if self.conditions.test(FieldCondition::RC_COMMANDS) {
                // Write roll, pitch and yaw first, these are signed values in the range [-500,500]
                let rc_commands = [current.rc_commands[0], current.rc_commands[1], current.rc_commands[2]];
                encoder.write_signed_vb_16_array(&rc_commands);

                // Write the throttle separately from the rest of the RC data as it's UNSIGNED.
                // Throttle lies in range [PWM_RANGE_MIN, PWM_RANGE_MAX], ie [1000, 2000]
                #[allow(clippy::cast_sign_loss)]
                encoder.write_unsigned_vb((current.rc_commands[MainState::THROTTLE] - self.min_throttle) as u32);
            }

            assert_i_field_encoding!("setpoint", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            if self.conditions.test(FieldCondition::SETPOINT) {
                // Write setpoint roll, pitch, yaw, and throttle
                encoder.write_signed_vb_16_array(&current.setpoints);
            }

            assert_i_field_encoding!("vbat_latest", FieldPredictor::VBATREF, FieldEncoding::NEG_14BIT);
            if self.conditions.test(FieldCondition::BATTERY_VOLTAGE) {
                //Our voltage is expected to decrease over the course of the flight, so store our difference from
                //the reference:
                // Write 14 bits even if the number is negative (which would otherwise result in 32 bits)
                encoder.write_unsigned_vb(u32::from(self.vbat_reference - current.battery_voltage) & 0x3FFF);
            }

            assert_i_field_encoding!("amperage_latest", FieldPredictor::ZERO, FieldEncoding::UNSIGNED_VB);
            if self.conditions.test(FieldCondition::BATTERY_CURRENT) {
                // 12bit value directly from ADC
                encoder.write_unsigned_vb_16(current.amperage);
            }

            #[cfg(feature = "magnetometer")]
            if self.condition_cache.test(FieldCondition::MAGNETOMETER) {
                encoder.write_signed_vb_16_array(&current.mag);
            }

            #[cfg(feature = "barometer")]
            if self.conditions.test(FieldCondition::BAROMETER) {
                encoder.write_signed_vb(current.baro_altitude);
            }

            #[cfg(feature = "rangefinder")]
            if self.condition_cache.test(FieldCondition::RANGEFINDER) {
                encoder.write_signed_vb(current.range_raw);
            }

            if self.conditions.test(FieldCondition::RSSI) {
                encoder.write_unsigned_vb_16(current.rssi);
            }

            assert_i_field_encoding!("gyro_adc", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            if self.conditions.test(FieldCondition::GYRO) {
                encoder.write_signed_vb_16_array(&current.gyro);
            }

            assert_i_field_encoding!("gyroUnfilt", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            if self.conditions.test(FieldCondition::GYRO_UNFILTERED) {
                encoder.write_signed_vb_16_array(&current.gyro_unfiltered);
            }

            assert_i_field_encoding!("accSmooth", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            if self.conditions.test(FieldCondition::ACC) {
                encoder.write_signed_vb_16_array(&current.acc);
            }

            assert_i_field_encoding!("imuQuaternion", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            if self.conditions.test(FieldCondition::ATTITUDE) {
                encoder.write_signed_vb_16_array(&current.orientation);
            }

            assert_i_field_encoding!("debug", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            if self.conditions.test(FieldCondition::DEBUG) {
                encoder.write_signed_vb_16_array(&current.debug);
            }

            assert_i_field_encoding!("motor", FieldPredictor::MIN_MOTOR, FieldEncoding::UNSIGNED_VB);
            if Logger::field_enabled(self.log_select_enabled, LogFieldSelect::MOTOR) {
                //Motors can be below minimum output when disarmed, but that doesn't happen much
                encoder.write_signed_vb_16(current.motor[0] - self.motor_output_min);

                //Motors tend to be similar to each other so use the first motor's value as a predicted of the others
                for ii in 1..self.motor_count {
                    encoder.write_signed_vb_16(current.motor[ii] - current.motor[0]);
                }
            }
            #[cfg(feature = "servos")]
            if self.condition_cache.test(FieldCondition::SERVOS) {
                let out: [i32; MainState::MAX_SUPPORTED_SERVO_COUNT] =
                    std::array::from_fn(|i| i32::from(current.servos[i]) - 1500);
                encoder.write_tag8_8svb(&out);
            }
            #[cfg(feature = "dshot_telemetry")]
            if Logger::field_enabled(self.log_select_enabled, LogFieldSelect::MOTOR) {
                for erpm in current.erpm {
                    encoder.write_signed_vb_16(erpm);
                }
            }
        }

        let ret = encoder.end_frame();

        // Rotate the state indices
        let new_current = self.main_state_index_pre_previous;
        self.main_state_index_pre_previous = self.main_state_index_previous;
        self.main_state_index_previous = self.main_state_index_current;
        self.main_state_index_current = new_current;
        // This is an i_frame, so there is no other pre_previous state, so we copy the previous state into the pre_previous state
        self.main_states[self.main_state_index_pre_previous] = self.main_states[self.main_state_index_previous];

        ret
    }

    /// Write a Predictor frame (p_frame).
    /// Note: the predictions are hard coded to match the values defined in BLACKBOX_MAIN_FIELDS:
    /// the code is made safe by asserting the p_encoding values.
    /// So this code and those definitions must be changed in tandem with each other.
    #[allow(clippy::too_many_lines)]
    pub fn log_p_frame(&mut self, encoder: &mut SliceWriter) -> usize {
        self.logged_any_frames = true;

        encoder.begin_frame(b'P');

        let current = &self.main_states[self.main_state_index_current];
        let previous = &self.main_states[self.main_state_index_previous];
        let pre_previous = &self.main_states[self.main_state_index_pre_previous];

        //No need to store iteration count since its delta is always 1

        // Since the difference between the difference between successive times will be nearly zero (due to consistent
        // loop time spacing), use second-order differences.
        assert_p_field_encoding!("loopIteration", FieldPredictor::INC, FieldEncoding::ZERO);
        encoder.write_unsigned_vb(current.time_us - 2 * previous.time_us + pre_previous.time_us);

        // if self.condition_cache.test(FieldCondition::GYRO_UNFILTERED) {
        assert_p_field_encoding!("axisP", FieldPredictor::PREVIOUS, FieldEncoding::SIGNED_VB);
        assert_p_field_encoding!("axisI", FieldPredictor::PREVIOUS, FieldEncoding::TAG2_3S32);
        assert_p_field_encoding!("axisD", FieldPredictor::PREVIOUS, FieldEncoding::SIGNED_VB);
        assert_p_field_encoding!("axisF", FieldPredictor::PREVIOUS, FieldEncoding::SIGNED_VB);
        assert_p_field_encoding!("axisS", FieldPredictor::PREVIOUS, FieldEncoding::SIGNED_VB);
        if self.conditions.test(FieldCondition::PID) {
            let deltas = [
                current.pid_p[0] - previous.pid_p[0],
                current.pid_p[1] - previous.pid_p[1],
                current.pid_p[2] - previous.pid_p[2],
            ];
            encoder.write_signed_vb_array(&deltas);

            // The PID I field changes very slowly, most of the time +-2, so use an encoding
            // that can pack all three fields into one byte in that situation.
            let deltas = [
                current.pid_i[0] - previous.pid_i[0],
                current.pid_i[1] - previous.pid_i[1],
                current.pid_i[2] - previous.pid_i[2],
            ];
            encoder.write_tag2_3s32(deltas);

            // The PID D term is frequently set to zero for yaw, which makes the result from the calculation
            // always zero. So don't bother recording D results when PID D terms are zero.
            if self.conditions.test(FieldCondition::PID_D_ROLL) {
                encoder.write_signed_vb(current.pid_d[0] - previous.pid_d[0]);
            }
            if self.conditions.test(FieldCondition::PID_D_PITCH) {
                encoder.write_signed_vb(current.pid_d[1] - previous.pid_d[1]);
            }
            if self.conditions.test(FieldCondition::PID_D_YAW) {
                encoder.write_signed_vb(current.pid_d[2] - previous.pid_d[2]);
            }

            if self.conditions.test(FieldCondition::PID_K) {
                let deltas = [
                    current.pid_k[0] - previous.pid_k[0],
                    current.pid_k[1] - previous.pid_k[1],
                    current.pid_k[2] - previous.pid_k[2],
                ];
                encoder.write_signed_vb_array(&deltas);
            }

            if self.conditions.test(FieldCondition::PID_S_ROLL) {
                encoder.write_signed_vb(current.pid_s[0] - previous.pid_s[0]);
            }
            if self.conditions.test(FieldCondition::PID_S_PITCH) {
                encoder.write_signed_vb(current.pid_s[1] - previous.pid_s[1]);
            }
            if self.conditions.test(FieldCondition::PID_S_YAW) {
                encoder.write_signed_vb(current.pid_s[2] - previous.pid_s[2]);
            }
        }

        // RC tends to stay the same or fairly small for many frames at a time, so use an encoding that
        assert_p_field_encoding!("rc_command", FieldPredictor::PREVIOUS, FieldEncoding::TAG8_4S16);
        if self.conditions.test(FieldCondition::RC_COMMANDS) {
            let deltas = [
                current.rc_commands[0] - previous.rc_commands[0],
                current.rc_commands[1] - previous.rc_commands[1],
                current.rc_commands[2] - previous.rc_commands[2],
                current.rc_commands[3] - previous.rc_commands[3],
            ];
            encoder.write_tag8_4s16(deltas);
        }
        assert_p_field_encoding!("setpoint", FieldPredictor::PREVIOUS, FieldEncoding::TAG8_4S16);
        if self.conditions.test(FieldCondition::SETPOINT) {
            let deltas = [
                current.setpoints[0] - previous.setpoints[0],
                current.setpoints[1] - previous.setpoints[1],
                current.setpoints[2] - previous.setpoints[2],
                current.setpoints[3] - previous.setpoints[3],
            ];
            encoder.write_tag8_4s16(deltas);
        }

        let mut deltas = <[i32; 8]>::default();
        //Check for sensors that are updated periodically (so deltas are normally zero)
        let mut optional_field_count = 0usize;

        if self.conditions.test(FieldCondition::BATTERY_VOLTAGE) {
            deltas[optional_field_count] = i32::from(current.battery_voltage - previous.battery_voltage);
            optional_field_count += 1;
        }

        if self.conditions.test(FieldCondition::BATTERY_CURRENT) {
            deltas[optional_field_count] = i32::from(current.amperage - previous.amperage);
            optional_field_count += 1;
        }

        #[cfg(feature = "magnetometer")]
        if self.condition_cache.test(FieldCondition::MAGNETOMETER) {
            for ii in 0..MainState::XYZ_AXIS_COUNT {
                deltas[optional_field_count] = i32::from(current.mag[ii] - previous.mag[ii]);
                optional_field_count += 1;
            }
        }

        #[cfg(feature = "barometer")]
        if self.conditions.test(FieldCondition::BAROMETER) {
            deltas[optional_field_count] = current.baro_altitude - previous.baro_altitude;
            optional_field_count += 1;
        }

        #[cfg(feature = "rangefinder")]
        if self.condition_cache.test(FieldCondition::RANGEFINDER) {
            deltas[optional_field_count] = current.range_raw - previous.range_raw;
            optional_field_count += 1;
        }

        if self.conditions.test(FieldCondition::RSSI) {
            deltas[optional_field_count] = i32::from(current.rssi - previous.rssi);
        }

        assert_p_field_encoding!("vbat_latest", FieldPredictor::PREVIOUS, FieldEncoding::TAG8_8SVB);
        assert_p_field_encoding!("amperage_latest", FieldPredictor::PREVIOUS, FieldEncoding::TAG8_8SVB);
        encoder.write_tag8_8svb(&deltas);

        // Since gyros, accelerometers and motors are noisy, base their predictions on the average of the history:
        assert_p_field_encoding!("gyro_adc", FieldPredictor::AVERAGE_2, FieldEncoding::SIGNED_VB);
        if self.conditions.test(FieldCondition::GYRO) {
            for ii in 0..MainState::XYZ_AXIS_COUNT {
                let predicted = i16::midpoint(previous.gyro[ii], pre_previous.gyro[ii]);
                encoder.write_signed_vb_16(current.gyro[ii] - predicted);
            }
        }
        assert_p_field_encoding!("gyroUnfilt", FieldPredictor::AVERAGE_2, FieldEncoding::SIGNED_VB);
        if self.conditions.test(FieldCondition::GYRO_UNFILTERED) {
            for ii in 0..MainState::XYZ_AXIS_COUNT {
                let predicted = i16::midpoint(previous.gyro_unfiltered[ii], pre_previous.gyro_unfiltered[ii]);
                encoder.write_signed_vb_16(current.gyro_unfiltered[ii] - predicted);
            }
        }
        assert_p_field_encoding!("accSmooth", FieldPredictor::AVERAGE_2, FieldEncoding::SIGNED_VB);
        if self.conditions.test(FieldCondition::ACC) {
            for ii in 0..MainState::XYZ_AXIS_COUNT {
                let predicted = i16::midpoint(previous.acc[ii], pre_previous.acc[ii]);
                encoder.write_signed_vb_16(current.acc[ii] - predicted);
            }
        }
        assert_p_field_encoding!("imuQuaternion", FieldPredictor::AVERAGE_2, FieldEncoding::SIGNED_VB);
        if self.conditions.test(FieldCondition::ATTITUDE) {
            for ii in 0..MainState::XYZ_AXIS_COUNT {
                let predicted = i16::midpoint(previous.orientation[ii], pre_previous.orientation[ii]);
                encoder.write_signed_vb_16(current.orientation[ii] - predicted);
            }
        }

        assert_p_field_encoding!("debug", FieldPredictor::AVERAGE_2, FieldEncoding::SIGNED_VB);
        if self.conditions.test(FieldCondition::DEBUG) {
            for ii in 0..MainState::DEBUG_VALUE_COUNT {
                let predicted = i16::midpoint(previous.debug[ii], pre_previous.debug[ii]);
                encoder.write_signed_vb_16(current.debug[ii] - predicted);
            }
        }

        assert_p_field_encoding!("motor", FieldPredictor::AVERAGE_2, FieldEncoding::SIGNED_VB);
        if Logger::field_enabled(self.log_select_enabled, LogFieldSelect::MOTOR) {
            for ii in 0..self.motor_count {
                let predicted = i16::midpoint(previous.motor[ii], pre_previous.motor[ii]);
                encoder.write_signed_vb_16(current.motor[ii] - predicted);
            }
        }

        #[cfg(feature = "servos")]
        if self.condition_cache.test(FieldCondition::SERVOS) {
            let servos: [i32; MainState::MAX_SUPPORTED_SERVO_COUNT] =
                core::array::from_fn(|ii| i32::from(current.servos[ii]) - 1500);
            encoder.write_tag8_8svb(&servos);
        }

        #[cfg(feature = "dshot_telemetry")]
        if Logger::field_enabled(self.log_select_enabled, LogFieldSelect::MOTOR_RPM) {
            for ii in 0..self.motor_count {
                encoder.write_signed_vb_16(current.erpm[ii] - previous.erpm[ii]);
            }
        }
        let ret = encoder.end_frame();

        // Rotate the state indices
        let new_current = self.main_state_index_pre_previous;
        self.main_state_index_pre_previous = self.main_state_index_previous;
        self.main_state_index_previous = self.main_state_index_current;
        self.main_state_index_current = new_current;

        ret
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused_results)]

    use super::*;

    #[test]
    fn i_encodings() {
        assert_i_field_encoding!("loopIteration", FieldPredictor::ZERO, FieldEncoding::UNSIGNED_VB);
        let mut blackbox = Logger::default();
        assert_eq!(0, blackbox.main_state_index_current);
        assert_eq!(1, blackbox.main_state_index_previous);
        assert_eq!(2, blackbox.main_state_index_pre_previous);
        let mut buffer = [0u8; 512];
        let mut encoder = SliceWriter { buffer: &mut buffer, pos: 0 };
        blackbox.log_i_frame(&mut encoder);
        assert_eq!(2, blackbox.main_state_index_current);
        assert_eq!(0, blackbox.main_state_index_previous);
        assert_eq!(1, blackbox.main_state_index_pre_previous);
    }
    #[test]
    fn p_encodings() {
        assert_p_field_encoding!("loopIteration", FieldPredictor::INC, FieldEncoding::ZERO);
        let mut blackbox = Logger::default();
        assert_eq!(0, blackbox.main_state_index_current);
        assert_eq!(1, blackbox.main_state_index_previous);
        assert_eq!(2, blackbox.main_state_index_pre_previous);

        let mut buffer = [0u8; 512];
        let mut encoder = SliceWriter { buffer: &mut buffer, pos: 0 };
        blackbox.log_p_frame(&mut encoder);
        assert_eq!(2, blackbox.main_state_index_current);
        assert_eq!(0, blackbox.main_state_index_previous);
        assert_eq!(1, blackbox.main_state_index_pre_previous);
    }
}
