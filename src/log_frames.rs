use crate::{
    BlackboxWriter, SliceEncoder,
    data::{BlackboxEvent, BlackboxEventId},
    field_definitions::{FieldCondition, FieldSelect},
    logger::Logger,
};

#[cfg(feature = "huffman")]
use crate::{huffman_encoder::HuffmanEncoder, huffman_table::HUFFMAN_TABLE};

#[cfg(feature = "servos")]
use crate::data::BlackboxMainData;

#[allow(unused)]
use crate::field_definitions::FieldPredictor; // used in macro_rules, so sometimes not visible to compiler

#[cfg(test)]
use crate::field_definitions::{FieldEncoding, MainFieldDefinition, SimpleFieldDefinition};

#[cfg(all(test, feature = "gps"))]
use crate::field_definitions::ConditionalFieldDefinition;

macro_rules! assert_i_field_encoding {
    ($name:expr, $expected_predict:expr, $expected_encode:expr) => {
        #[cfg(test)]
        {
            let field = MainFieldDefinition::find_main_field_by_name($name).expect(concat!("Field not found: ", $name));
            assert_eq!(field.i_predict, $expected_predict, "I PREDICT mismatch for field: \"{}\"", $name);
            assert_eq!(field.i_encode, $expected_encode, "I ENCODE mismatch for field: \"{}\"", $name);
        }
    };
}

macro_rules! assert_p_field_encoding {
    ($name:expr, $expected_predict:expr, $expected_encode:expr) => {
        #[cfg(test)]
        {
            let field = MainFieldDefinition::find_main_field_by_name($name).expect(concat!("Field not found: ", $name));
            assert_eq!(field.p_predict, $expected_predict, "P PREDICT mismatch for field: \"{}\"", $name);
            assert_eq!(field.p_encode, $expected_encode, "P ENCODE mismatch for field: \"{}\"", $name);
        }
    };
}

macro_rules! assert_s_field_encoding {
    ($name:expr, $expected_predict:expr, $expected_encode:expr) => {
        #[cfg(test)]
        {
            let field = SimpleFieldDefinition::find_s_field_by_name($name).expect(concat!("Field not found: ", $name));
            assert_eq!(field.predict, $expected_predict, "PREDICT mismatch for field: \"{}\"", $name);
            assert_eq!(field.encode, $expected_encode, "ENCODE mismatch for field: \"{}\"", $name);
        }
    };
}

#[cfg(feature = "gps")]
macro_rules! assert_h_field_encoding {
    ($name:expr, $expected_predict:expr, $expected_encode:expr) => {
        #[cfg(test)]
        {
            let field = SimpleFieldDefinition::find_h_field_by_name($name).expect(concat!("Field not found: ", $name));
            assert_eq!(field.predict, $expected_predict, "PREDICT mismatch for field: \"{}\"", $name);
            assert_eq!(field.encode, $expected_encode, "ENCODE mismatch for field: \"{}\"", $name);
        }
    };
}

#[cfg(feature = "gps")]
macro_rules! assert_g_field_encoding {
    ($name:expr, $expected_predict:expr, $expected_encode:expr) => {
        #[cfg(test)]
        {
            let field =
                ConditionalFieldDefinition::find_g_field_by_name($name).expect(concat!("Field not found: ", $name));
            assert_eq!(field.predict, $expected_predict, "PREDICT mismatch for field: \"{}\"", $name);
            assert_eq!(field.encode, $expected_encode, "ENCODE mismatch for field: \"{}\"", $name);
        }
    };
}

impl Logger {
    /// Log event: `e_frame`. Written immediately to log when event occurs.
    pub fn log_e_frame(&mut self, encoder: &mut SliceEncoder, event: BlackboxEvent) {
        encoder.begin_frame(b'E');

        match event {
            BlackboxEvent::SyncBeep(time) => {
                encoder.write_byte(BlackboxEventId::SYNC_BEEP);
                encoder.write_unsigned_vb(time);
            }
            BlackboxEvent::InflightAdjustment(new_value, new_value_f32, adjustment, is_float) => {
                encoder.write_byte(BlackboxEventId::INFLIGHT_ADJUSTMENT);
                encoder.write_signed_vb(0);
                if is_float {
                    const IS_F32_FLAG: u8 = 128;
                    encoder.write_byte(adjustment + IS_F32_FLAG);
                    encoder.write_f32(new_value_f32);
                } else {
                    encoder.write_byte(adjustment);
                    encoder.write_signed_vb(new_value);
                }
            }
            BlackboxEvent::Disarm(reason) => {
                encoder.write_byte(BlackboxEventId::DISARM);
                encoder.write_unsigned_vb(reason);
            }
            BlackboxEvent::LoggingResume(iteration, time) => {
                encoder.write_byte(BlackboxEventId::LOGGING_RESUME);
                encoder.write_unsigned_vb(iteration);
                encoder.write_unsigned_vb(time);
            }
            BlackboxEvent::FlightMode(flags, previous_flags) => {
                encoder.write_byte(BlackboxEventId::FLIGHT_MODE);
                encoder.write_unsigned_vb(flags);
                encoder.write_unsigned_vb(previous_flags);
            }
            BlackboxEvent::LogEnd => {
                encoder.write_byte(BlackboxEventId::LOG_END);
                encoder.write_str("End of log");
                encoder.write_byte(0);
                // TODO:
            }
            _ => {}
        }
        encoder.end_frame();
    }

    /// Log slow frame: `s_frame`.
    pub fn log_s_frame(&mut self, encoder: &mut SliceEncoder) {
        self.s_frame_index = 0;

        encoder.begin_frame(b'S');

        assert_s_field_encoding!("flight_mode_flags", FieldPredictor::Zero, FieldEncoding::UnsignedVb);
        encoder.write_unsigned_vb(self.slow_data.flight_mode_flags);

        assert_s_field_encoding!("state_flags", FieldPredictor::Zero, FieldEncoding::UnsignedVb);
        encoder.write_unsigned_vb(u32::from(self.slow_data.gps_state_flags));

        assert_s_field_encoding!("failsafe_phase", FieldPredictor::Zero, FieldEncoding::Tag2_3S32);
        assert_s_field_encoding!("rx_signal_received", FieldPredictor::Zero, FieldEncoding::Tag2_3S32);
        assert_s_field_encoding!("rx_flight_channel_is_valid", FieldPredictor::Zero, FieldEncoding::Tag2_3S32);
        // Most of the time these three values will be able to pack into one byte.
        let values = [
            i32::from(self.slow_data.failsafe_phase),
            i32::from(self.slow_data.rx_signal_received),
            i32::from(self.slow_data.rx_flight_channel_is_valid),
        ];
        encoder.write_tag2_3s32(values);

        encoder.end_frame();
    }

    /// GPS home frame: `h_frame`.
    #[cfg(feature = "gps")]
    pub fn log_h_frame(&mut self, encoder: &mut SliceEncoder) {
        self.has_new_gps_data = false;

        encoder.begin_frame(b'H');

        assert_h_field_encoding!("GPS_home", FieldPredictor::Zero, FieldEncoding::SignedVb);
        encoder.write_signed_vb(self.gps_home.latitude_degrees_x1e7);
        encoder.write_signed_vb(self.gps_home.longitude_degrees_x1e7);
        // log altitude in increments of 0.1m
        encoder.write_signed_vb(self.gps_home.altitude_cm / 10);

        // TODO: convert gps time to unix time
        assert_h_field_encoding!("GPS_home_epoch", FieldPredictor::Zero, FieldEncoding::UnsignedVb);
        encoder.write_unsigned_vb(0);

        encoder.end_frame();
    }

    /// GPS frame: `g_frame`. Written at a frequency of about 10Hz.
    #[cfg(feature = "gps")]
    pub fn log_g_frame(&mut self, encoder: &mut SliceEncoder, current_time_us: u32) {
        self.has_new_gps_data = false;

        encoder.begin_frame(b'G');

        // If we're logging every frame, then a GPS frame always appears just after a frame with the
        // current_time timestamp in the log, so the reader can just use that timestamp for the GPS frame.
        // If we're not logging every frame, we need to store the time of this GPS frame.
        assert_g_field_encoding!("time", FieldPredictor::LastMainFrameTime, FieldEncoding::UnsignedVb);
        if self.conditions.test(FieldCondition::NOT_LOGGING_EVERY_FRAME) {
            // Predict the time of the last frame in the main log
            encoder.write_unsigned_vb(current_time_us - self.main_data[0].time_us);
        }

        assert_g_field_encoding!("GPS_numSat", FieldPredictor::Zero, FieldEncoding::UnsignedVb);
        encoder.write_unsigned_vb(u32::from(self.gps_data.satellite_count));

        assert_g_field_encoding!("GPS_coord", FieldPredictor::HomeCoord, FieldEncoding::SignedVb);
        encoder.write_signed_vb(self.gps_data.position.latitude_degrees_x1e7 - self.gps_home.latitude_degrees_x1e7);
        encoder.write_signed_vb(self.gps_data.position.longitude_degrees_x1e7 - self.gps_home.longitude_degrees_x1e7);

        // log altitude in increments of 0.1m
        assert_g_field_encoding!("GPS_altitude", FieldPredictor::Zero, FieldEncoding::SignedVb);
        encoder.write_signed_vb(self.gps_data.position.altitude_cm / 10);

        //if self.config.gps_use_3d_speed {
        //    encoder.write_unsigned_vb(self.gps_data.speed3d_cmps as u32);
        //} else {
        assert_g_field_encoding!("GPS_speed", FieldPredictor::Zero, FieldEncoding::UnsignedVb);
        #[allow(clippy::cast_sign_loss)]
        encoder.write_unsigned_vb(self.gps_data.ground_speed_cmps as u32);
        //}

        assert_g_field_encoding!("GPS_ground_course", FieldPredictor::Zero, FieldEncoding::UnsignedVb);
        #[allow(clippy::cast_sign_loss)]
        encoder.write_unsigned_vb(self.gps_data.ground_course_degrees_x10 as u32);

        assert_g_field_encoding!("GPS_velned", FieldPredictor::Zero, FieldEncoding::SignedVb);
        encoder.write_signed_vb_16(self.gps_data.velocity_north_cmps);
        encoder.write_signed_vb_16(self.gps_data.velocity_east_cmps);
        encoder.write_signed_vb_16(self.gps_data.velocity_down_cmps);

        assert_g_field_encoding!("GPS_time", FieldPredictor::Zero, FieldEncoding::UnsignedVb);
        encoder.write_unsigned_vb(self.gps_data.time_of_week_ms);
        encoder.end_frame();
    }

    /// Write an Intra frame (`i_frame`).
    /// Also known as a key frame.
    #[allow(clippy::too_many_lines)]
    pub fn log_i_frame(&mut self, encoder: &mut SliceEncoder) {
        self.main_data_current_idx = 0;
        self.main_data_previous_idx = 1;
        self.main_data_pre_previous_idx = 2;
        let current = &self.main_data[0];

        encoder.begin_frame(b'I');

        assert_i_field_encoding!("loopIteration", FieldPredictor::Zero, FieldEncoding::UnsignedVb);
        encoder.write_unsigned_vb(self.iteration);

        assert_i_field_encoding!("time", FieldPredictor::Zero, FieldEncoding::UnsignedVb);
        encoder.write_unsigned_vb(current.time_us);

        assert_i_field_encoding!("axisP", FieldPredictor::Zero, FieldEncoding::SignedVb);
        assert_i_field_encoding!("axisI", FieldPredictor::Zero, FieldEncoding::SignedVb);
        assert_i_field_encoding!("axisD", FieldPredictor::Zero, FieldEncoding::SignedVb);
        assert_i_field_encoding!("axisF", FieldPredictor::Zero, FieldEncoding::SignedVb);
        assert_i_field_encoding!("axisS", FieldPredictor::Zero, FieldEncoding::SignedVb);
        if self.conditions.test(FieldCondition::PID) {
            encoder.write_signed_vb_array(&current.pid_p);
            assert_i_field_encoding!("axisI", FieldPredictor::Zero, FieldEncoding::SignedVb);
            encoder.write_signed_vb_array(&current.pid_i);

            if self.conditions.test(FieldCondition::PID_D_ROLL) {
                encoder.write_signed_vb(current.pid_d[0]);
            }
            if self.conditions.test(FieldCondition::PID_D_PITCH) {
                encoder.write_signed_vb(current.pid_d[1]);
            }
            if self.conditions.test(FieldCondition::PID_D_YAW) {
                encoder.write_signed_vb(current.pid_d[2]);
            }

            if self.conditions.test(FieldCondition::PID_K) {
                encoder.write_signed_vb_array(&current.pid_k);
            }

            if self.conditions.test(FieldCondition::PID_S_ROLL) {
                encoder.write_signed_vb(current.pid_s[0]);
            }
            if self.conditions.test(FieldCondition::PID_S_PITCH) {
                encoder.write_signed_vb(current.pid_s[1]);
            }
            if self.conditions.test(FieldCondition::PID_S_YAW) {
                encoder.write_signed_vb(current.pid_s[2]);
            }
        }
        assert_i_field_encoding!("rcCommand", FieldPredictor::Zero, FieldEncoding::SignedVb);
        if self.conditions.test(FieldCondition::RC_COMMANDS) {
            // Write roll, pitch and yaw first, these are signed values in the range [-500,500]
            let rc_commands = [
                current.rc_commands[0].cast_signed(),
                current.rc_commands[1].cast_signed(),
                current.rc_commands[2].cast_signed(),
            ];
            encoder.write_signed_vb_16_array(&rc_commands);

            // Write the throttle separately from the rest of the RC data as it's UNSIGNED.
            // Throttle lies in range [PWM_RANGE_MIN, PWM_RANGE_MAX], ie [1000, 2000]
            encoder.write_unsigned_vb(u32::from(current.rc_commands[3]));
        }

        assert_i_field_encoding!("setpoint", FieldPredictor::Zero, FieldEncoding::SignedVb);
        if self.conditions.test(FieldCondition::SETPOINT) {
            // Write setpoint roll, pitch, yaw, and throttle
            encoder.write_signed_vb_16_array(&current.setpoints);
        }

        assert_i_field_encoding!("vbatLatest", FieldPredictor::VBatRef, FieldEncoding::Neg14bit);
        if self.conditions.test(FieldCondition::BATTERY_VOLTAGE) {
            // Our voltage is expected to decrease over the course of the flight, so store our difference from the reference.
            // Write 14 bits even if the number is negative (which would otherwise result in 32 bits)
            encoder.write_unsigned_vb(u32::from(self.vbat_reference - current.battery_voltage) & 0x3FFF);
        }

        assert_i_field_encoding!("amperageLatest", FieldPredictor::Zero, FieldEncoding::SignedVb);
        if self.conditions.test(FieldCondition::BATTERY_CURRENT) {
            // 12bit value directly from ADC
            encoder.write_signed_vb_16(current.battery_current);
        }

        assert_i_field_encoding!("BaroAlt", FieldPredictor::Zero, FieldEncoding::SignedVb);
        if self.conditions.test(FieldCondition::BAROMETER) {
            encoder.write_signed_vb(current.barometer_altitude);
        }

        #[cfg(feature = "rangefinder")]
        assert_i_field_encoding!("surfaceRaw", FieldPredictor::Zero, FieldEncoding::SignedVb);
        #[cfg(feature = "rangefinder")]
        if self.conditions.test(FieldCondition::RANGEFINDER) {
            encoder.write_signed_vb(current.range_raw);
        }

        assert_i_field_encoding!("rssi", FieldPredictor::Zero, FieldEncoding::UnsignedVb);
        if self.conditions.test(FieldCondition::RSSI) {
            encoder.write_unsigned_vb_16(current.rssi);
        }

        #[cfg(feature = "magnetometer")]
        assert_i_field_encoding!("magADC", FieldPredictor::Zero, FieldEncoding::SignedVb);
        #[cfg(feature = "magnetometer")]
        if self.conditions.test(FieldCondition::MAGNETOMETER) {
            encoder.write_signed_vb_16_array(&current.mag);
        }

        assert_i_field_encoding!("gyroADC", FieldPredictor::Zero, FieldEncoding::SignedVb);
        if self.conditions.test(FieldCondition::GYRO) {
            encoder.write_signed_vb_16_array(&current.gyro);
        }

        assert_i_field_encoding!("gyroUnfilt", FieldPredictor::Zero, FieldEncoding::SignedVb);
        if self.conditions.test(FieldCondition::GYRO_UNFILTERED) {
            encoder.write_signed_vb_16_array(&current.gyro_unfiltered);
        }

        assert_i_field_encoding!("accSmooth", FieldPredictor::Zero, FieldEncoding::SignedVb);
        if self.conditions.test(FieldCondition::ACC) {
            encoder.write_signed_vb_16_array(&current.acc);
        }

        assert_i_field_encoding!("imuQuaternion", FieldPredictor::Zero, FieldEncoding::SignedVb);
        if self.conditions.test(FieldCondition::ATTITUDE) {
            encoder.write_signed_vb_16_array(&current.orientation);
        }

        #[cfg(feature = "debug")]
        assert_i_field_encoding!("debug", FieldPredictor::Zero, FieldEncoding::SignedVb);
        #[cfg(feature = "debug")]
        if self.conditions.test(FieldCondition::DEBUG) {
            encoder.write_signed_vb_16_array(&current.debug);
        }

        assert_i_field_encoding!("motor", FieldPredictor::MinMotor, FieldEncoding::SignedVb);
        if Logger::field_enabled(self.enabled_fields, FieldSelect::MOTOR) {
            // Motors can be below minimum output when disarmed, but that doesn't happen much
            encoder.write_signed_vb_16(current.motor[0].wrapping_sub(self.sys_info.motor_output_min.cast_signed()));

            // Motors tend to be similar to each other so use the first motor's value as a predicted of the others
            for ii in 1..self.motor_count {
                encoder.write_signed_vb_16(current.motor[ii].wrapping_sub(current.motor[0]));
            }
        }
        #[cfg(feature = "dshot_telemetry")]
        assert_i_field_encoding!("eRPM", FieldPredictor::Zero, FieldEncoding::UnsignedVb);
        #[cfg(feature = "dshot_telemetry")]
        if Logger::field_enabled(self.enabled_fields, FieldSelect::MOTOR_RPM) {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            for erpm_d2 in current.erpm_d2 {
                encoder.write_unsigned_vb_16((erpm_d2 as u16) * 2);
            }
        }
        #[cfg(feature = "servos")]
        if self.conditions.test(FieldCondition::SERVOS) {
            let out: [i32; BlackboxMainData::MAX_SUPPORTED_SERVO_COUNT] =
                core::array::from_fn(|i| i32::from(current.servos[i]) - 1500);
            encoder.write_tag8_8svb(&out);
        }

        encoder.end_frame();

        // This is an i_frame, so there is no other previous data, so we copy the current data into the pre_previous data.
        self.main_data[2] = self.main_data[0];
        self.main_data[1] = self.main_data[0];
    }

    /// Write a Predictor frame (`p_frame`).
    /// Also known as an inter frame.
    /// Note: the predictions are hard coded to match the values defined in `BLACKBOX_MAIN_FIELDS`:
    /// the code is made safe by asserting the `p_encoding` values.
    /// So this code and those definitions must be changed in tandem with each other.
    #[allow(clippy::too_many_lines)]
    pub fn log_p_frame(&mut self, encoder: &mut SliceEncoder) -> usize {
        let current = &self.main_data[self.main_data_current_idx];
        let previous = &self.main_data[self.main_data_previous_idx];
        let pre_previous = &self.main_data[self.main_data_pre_previous_idx];

        let p_frame_start_pos = encoder.pos;
        encoder.begin_frame(b'P');

        // Don't store store iteration when using FieldEncoding::NULL
        assert_p_field_encoding!("loopIteration", FieldPredictor::Inc, FieldEncoding::Null);

        // Since the difference between the difference between successive times will be nearly zero (due to consistent
        // loop time spacing), use second-order differences.
        assert_p_field_encoding!("time", FieldPredictor::StraightLine, FieldEncoding::SignedVb);
        let time: i64 = i64::from(current.time_us) - 2 * i64::from(previous.time_us) + i64::from(pre_previous.time_us);
        #[allow(clippy::cast_possible_truncation)]
        encoder.write_signed_vb(time as i32);

        assert_p_field_encoding!("axisP", FieldPredictor::Previous, FieldEncoding::SignedVb);
        assert_p_field_encoding!("axisI", FieldPredictor::Previous, FieldEncoding::Tag2_3S32);
        assert_p_field_encoding!("axisD", FieldPredictor::Previous, FieldEncoding::SignedVb);
        assert_p_field_encoding!("axisF", FieldPredictor::Previous, FieldEncoding::SignedVb);
        assert_p_field_encoding!("axisS", FieldPredictor::Previous, FieldEncoding::SignedVb);
        if self.conditions.test(FieldCondition::PID) {
            let deltas = [
                current.pid_p[0].wrapping_sub(previous.pid_p[0]),
                current.pid_p[1].wrapping_sub(previous.pid_p[1]),
                current.pid_p[2].wrapping_sub(previous.pid_p[2]),
            ];
            encoder.write_signed_vb_array(&deltas);

            // The PID I field changes very slowly, most of the time +-2, so use an encoding
            // that can pack all three fields into one byte in that situation.
            let deltas = [
                current.pid_i[0].wrapping_sub(previous.pid_i[0]),
                current.pid_i[1].wrapping_sub(previous.pid_i[1]),
                current.pid_i[2].wrapping_sub(previous.pid_i[2]),
            ];
            encoder.write_tag2_3s32(deltas);

            // The PID D term is frequently set to zero for yaw, which makes the result from the calculation always zero.
            // So only record D values when explicitly asked to do so.
            if self.conditions.test(FieldCondition::PID_D_ROLL) {
                encoder.write_signed_vb(current.pid_d[0].wrapping_sub(previous.pid_d[0]));
            }
            if self.conditions.test(FieldCondition::PID_D_PITCH) {
                encoder.write_signed_vb(current.pid_d[1].wrapping_sub(previous.pid_d[1]));
            }
            if self.conditions.test(FieldCondition::PID_D_YAW) {
                encoder.write_signed_vb(current.pid_d[2].wrapping_sub(previous.pid_d[2]));
            }

            // K 'kick' terms, known as feedforward in Betaflight.
            if self.conditions.test(FieldCondition::PID_K) {
                let deltas = [
                    current.pid_k[0].wrapping_sub(previous.pid_k[0]),
                    current.pid_k[1].wrapping_sub(previous.pid_k[1]),
                    current.pid_k[2].wrapping_sub(previous.pid_k[2]),
                ];
                encoder.write_signed_vb_array(&deltas);
            }

            if self.conditions.test(FieldCondition::PID_S_ROLL) {
                encoder.write_signed_vb(current.pid_s[0].wrapping_sub(previous.pid_s[0]));
            }
            if self.conditions.test(FieldCondition::PID_S_PITCH) {
                encoder.write_signed_vb(current.pid_s[1].wrapping_sub(previous.pid_s[1]));
            }
            if self.conditions.test(FieldCondition::PID_S_YAW) {
                encoder.write_signed_vb(current.pid_s[2].wrapping_sub(previous.pid_s[2]));
            }
        }

        // RC tends to stay the same or fairly small for many frames at a time, so use an encoding that reflects that.
        assert_p_field_encoding!("rcCommand", FieldPredictor::Previous, FieldEncoding::Tag8_4S16);
        if self.conditions.test(FieldCondition::RC_COMMANDS) {
            let deltas = [
                current.rc_commands[0].wrapping_sub(previous.rc_commands[0]).cast_signed(),
                current.rc_commands[1].wrapping_sub(previous.rc_commands[1]).cast_signed(),
                current.rc_commands[2].wrapping_sub(previous.rc_commands[2]).cast_signed(),
                current.rc_commands[3].wrapping_sub(previous.rc_commands[3]).cast_signed(),
            ];
            encoder.write_tag8_4s16(deltas);
        }
        assert_p_field_encoding!("setpoint", FieldPredictor::Previous, FieldEncoding::Tag8_4S16);
        if self.conditions.test(FieldCondition::SETPOINT) {
            let deltas = [
                current.setpoints[0].wrapping_sub(previous.setpoints[0]),
                current.setpoints[1].wrapping_sub(previous.setpoints[1]),
                current.setpoints[2].wrapping_sub(previous.setpoints[2]),
                current.setpoints[3].wrapping_sub(previous.setpoints[3]),
            ];
            encoder.write_tag8_4s16(deltas);
        }

        // Check for sensors that are updated periodically (so deltas are normally zero)
        let mut deltas = [0i32; 8];
        let mut tag8_field_count = 0_usize;

        assert_p_field_encoding!("vbatLatest", FieldPredictor::Previous, FieldEncoding::Tag8_8SVb);
        if self.conditions.test(FieldCondition::BATTERY_VOLTAGE) {
            deltas[tag8_field_count] = i32::from(current.battery_voltage.wrapping_sub(previous.battery_voltage));
            tag8_field_count += 1;
        }
        assert_p_field_encoding!("amperageLatest", FieldPredictor::Previous, FieldEncoding::Tag8_8SVb);
        if self.conditions.test(FieldCondition::BATTERY_CURRENT) {
            deltas[tag8_field_count] = i32::from(current.battery_current.wrapping_sub(previous.battery_current));
            tag8_field_count += 1;
        }
        assert_p_field_encoding!("BaroAlt", FieldPredictor::Previous, FieldEncoding::Tag8_8SVb);
        if self.conditions.test(FieldCondition::BAROMETER) {
            deltas[tag8_field_count] = current.barometer_altitude.wrapping_sub(previous.barometer_altitude);
            tag8_field_count += 1;
        }
        #[cfg(feature = "rangefinder")]
        assert_p_field_encoding!("surfaceRaw", FieldPredictor::Previous, FieldEncoding::Tag8_8SVb);
        #[cfg(feature = "rangefinder")]
        if self.conditions.test(FieldCondition::RANGEFINDER) {
            deltas[tag8_field_count] = current.range_raw.wrapping_sub(previous.range_raw);
            tag8_field_count += 1;
        }
        assert_p_field_encoding!("rssi", FieldPredictor::Previous, FieldEncoding::Tag8_8SVb);
        if self.conditions.test(FieldCondition::RSSI) {
            deltas[tag8_field_count] = i32::from(current.rssi.wrapping_sub(previous.rssi));
            tag8_field_count += 1;
        }
        #[cfg(feature = "magnetometer")]
        assert_p_field_encoding!("magADC", FieldPredictor::Previous, FieldEncoding::Tag8_8SVb);
        #[cfg(feature = "magnetometer")]
        if self.conditions.test(FieldCondition::MAGNETOMETER) {
            for (&current_mag, &previous_mag) in current.mag.iter().zip(previous.mag.iter()) {
                deltas[tag8_field_count] = i32::from(current_mag.wrapping_sub(previous_mag));
                tag8_field_count += 1;
            }
        }

        if tag8_field_count > 0 {
            encoder.write_tag8_8svb(&deltas);
        }

        // Since gyros, accelerometers and motors are noisy, base their predictions on the average of the history:
        assert_p_field_encoding!("gyroADC", FieldPredictor::Previous, FieldEncoding::SignedVb);
        if self.conditions.test(FieldCondition::GYRO) {
            for (&current_gyro, &previous_gyro) in current.gyro.iter().zip(&previous.gyro) {
                encoder.write_signed_vb_16(current_gyro - previous_gyro);
            }
        }
        assert_p_field_encoding!("gyroUnfilt", FieldPredictor::Average2, FieldEncoding::SignedVb);
        if self.conditions.test(FieldCondition::GYRO_UNFILTERED) {
            for ((&current_gyro, &previous_gyro), &pre_previous_gyro) in
                current.gyro_unfiltered.iter().zip(&previous.gyro_unfiltered).zip(&pre_previous.gyro_unfiltered)
            {
                let predicted = i16::midpoint(previous_gyro, pre_previous_gyro);
                encoder.write_signed_vb_16(current_gyro.wrapping_sub(predicted));
            }
        }
        assert_p_field_encoding!("accSmooth", FieldPredictor::Average2, FieldEncoding::SignedVb);
        if self.conditions.test(FieldCondition::ACC) {
            for ((&current_acc, &previous_acc), &pre_previous_acc) in
                current.acc.iter().zip(&previous.acc).zip(&pre_previous.acc)
            {
                let predicted = i16::midpoint(previous_acc, pre_previous_acc);
                encoder.write_signed_vb_16(current_acc.wrapping_sub(predicted));
            }
        }
        assert_p_field_encoding!("imuQuaternion", FieldPredictor::Average2, FieldEncoding::SignedVb);
        if self.conditions.test(FieldCondition::ATTITUDE) {
            for ((&current_orientation, &previous_orientation), &pre_previous_orientation) in
                current.orientation.iter().zip(&previous.orientation).zip(&pre_previous.orientation)
            {
                let predicted = i16::midpoint(previous_orientation, pre_previous_orientation);
                encoder.write_signed_vb_16(current_orientation.wrapping_sub(predicted));
            }
        }

        #[cfg(feature = "debug")]
        assert_p_field_encoding!("debug", FieldPredictor::Average2, FieldEncoding::SignedVb);
        #[cfg(feature = "debug")]
        if self.conditions.test(FieldCondition::DEBUG) {
            for ((&current_debug, &previous_debug), &pre_previous_debug) in
                current.debug.iter().zip(&previous.debug).zip(&pre_previous.debug)
            {
                let predicted = i16::midpoint(previous_debug, pre_previous_debug);
                encoder.write_signed_vb_16(current_debug.wrapping_sub(predicted));
            }
        }
        assert_p_field_encoding!("motor", FieldPredictor::Average2, FieldEncoding::SignedVb);
        if Logger::field_enabled(self.enabled_fields, FieldSelect::MOTOR) {
            for ((&current_motor, &previous_motor), &pre_previous_motor) in current.motor[..self.motor_count]
                .iter()
                .zip(&previous.motor[..self.motor_count])
                .zip(&pre_previous.motor[..self.motor_count])
            {
                let predicted = i16::midpoint(previous_motor, pre_previous_motor);
                encoder.write_signed_vb_16(current_motor.wrapping_sub(predicted));
            }
        }
        #[cfg(feature = "dshot_telemetry")]
        assert_p_field_encoding!("eRPM", FieldPredictor::Previous, FieldEncoding::SignedVb);
        #[cfg(feature = "dshot_telemetry")]
        if Logger::field_enabled(self.enabled_fields, FieldSelect::MOTOR_RPM) {
            for (&current_erpm_d2, &previous_erpm_d2) in
                current.erpm_d2[..self.motor_count].iter().zip(&previous.erpm_d2[..self.motor_count])
            {
                encoder.write_signed_vb_16(current_erpm_d2.wrapping_sub(previous_erpm_d2) * 2);
            }
        }

        #[cfg(feature = "servos")]
        if self.conditions.test(FieldCondition::SERVOS) {
            let servos: [i32; BlackboxMainData::MAX_SUPPORTED_SERVO_COUNT] =
                core::array::from_fn(|ii| i32::from(current.servos[ii]) - 1500);
            encoder.write_tag8_8svb(&servos);
        }
        encoder.end_frame();

        // Rotate the saved data.
        //self.main_data[2] = self.main_data[1];
        //self.main_data[1] = self.main_data[0];
        let pre_previous_idx = self.main_data_pre_previous_idx;
        self.main_data_pre_previous_idx = self.main_data_previous_idx;
        self.main_data_previous_idx = self.main_data_current_idx;
        self.main_data_current_idx = pre_previous_idx;

        p_frame_start_pos
    }

    /// Convert a `p_frame` to a Huffman encoded `q_frame`.
    /// If there are any errors in the conversion, then we just return false, and leave the `p_frame` intact.
    #[cfg(feature = "huffman")]
    pub fn try_convert_p_frame_to_q_frame(&mut self, encoder: &mut SliceEncoder, p_frame_start_pos: usize) -> bool {
        let p_frame_length = encoder.pos - p_frame_start_pos;
        let Ok(huffman_encoder) =
            HuffmanEncoder::<{ Logger::Q_FRAME_MAX_INPUT_LENGTH }>::new(self.q_frame_buffer.as_mut_slice())
        else {
            // If we can't create the HuffmanEncoder, then we just return false, and leave the `p_frame` intact.
            return false;
        };

        // Skip over the initial 'P' character.
        if let Some(slice) = encoder.get_slice(p_frame_start_pos + 1, p_frame_length - 1)
            && let Ok(q_frame_length) = huffman_encoder.try_compress(slice)
            && q_frame_length < p_frame_length
        {
            // Set the frame type to Q.
            encoder.buffer[p_frame_start_pos] = b'Q';
            // Copy the q_frame_buffer into the encoder buffer.
            encoder.buffer[p_frame_start_pos + 1..p_frame_start_pos + 1 + q_frame_length]
                .copy_from_slice(&self.q_frame_buffer.as_slice()[..q_frame_length]);
            // Set the encoder position.
            encoder.pos = p_frame_start_pos + q_frame_length + 1;
            return true;
        }
        false
    }

    /// Log Huffman table frame: `t_frame`.
    /// T followed by 768 bytes of the table, that is 256 3-byte triplets of len u8 and code u16-little-endian.
    #[cfg(feature = "huffman")]
    pub fn log_t_frame(&mut self, encoder: &mut SliceEncoder) {
        encoder.begin_frame(b'T');

        for huffman_code in HUFFMAN_TABLE {
            #[allow(clippy::cast_possible_truncation)]
            encoder.write_byte(huffman_code.len as u8);
            for code in huffman_code.code.to_be_bytes() {
                encoder.write_byte(code);
            }
        }
        encoder.end_frame();
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[cfg(feature = "gps")]
    #[test]
    fn e_encodings() {
        let mut blackbox = Logger::default();

        let mut buffer = [0u8; 512];
        let mut encoder = SliceEncoder { buffer: &mut buffer, pos: 0 };

        blackbox.log_e_frame(&mut encoder, BlackboxEvent::LogEnd);
    }

    #[cfg(feature = "gps")]
    #[test]
    fn g_encodings() {
        let mut blackbox = Logger::default();

        let mut buffer = [0u8; 512];
        let mut encoder = SliceEncoder { buffer: &mut buffer, pos: 0 };

        // Ensures the assert_g_field_encoding macros are run.
        blackbox.log_g_frame(&mut encoder, 123_456_789);
    }

    #[cfg(feature = "gps")]
    #[test]
    fn h_encodings() {
        let mut blackbox = Logger::default();

        let mut buffer = [0u8; 512];
        let mut encoder = SliceEncoder { buffer: &mut buffer, pos: 0 };

        // Ensures the assert_s_field_encoding macros are run.
        blackbox.log_h_frame(&mut encoder);
    }

    #[test]
    fn s_encodings() {
        let mut blackbox = Logger::default();

        let mut buffer = [0u8; 512];
        let mut encoder = SliceEncoder { buffer: &mut buffer, pos: 0 };

        // Ensures the assert_s_field_encoding macros are run.
        blackbox.log_s_frame(&mut encoder);
    }

    #[test]
    fn i_encodings() {
        assert_i_field_encoding!("loopIteration", FieldPredictor::Zero, FieldEncoding::UnsignedVb);
        assert_i_field_encoding!("time", FieldPredictor::Zero, FieldEncoding::UnsignedVb);

        let mut blackbox = Logger::default();
        blackbox.main_data[0].time_us = 3;
        blackbox.main_data[1].time_us = 2;
        blackbox.main_data[2].time_us = 1;

        let mut buffer = [0u8; 512];
        let mut encoder = SliceEncoder { buffer: &mut buffer, pos: 0 };

        // Ensures the assert_i_field_encoding macros are run.
        blackbox.log_i_frame(&mut encoder);

        assert_eq!(3, blackbox.main_data[0].time_us);
        assert_eq!(3, blackbox.main_data[1].time_us);
        assert_eq!(3, blackbox.main_data[2].time_us);

        blackbox.main_data[0].time_us = 4;
        blackbox.log_i_frame(&mut encoder);
        assert_eq!(4, blackbox.main_data[0].time_us);
        assert_eq!(4, blackbox.main_data[1].time_us);
        assert_eq!(4, blackbox.main_data[2].time_us);
    }

    #[test]
    fn p_encodings() {
        assert_p_field_encoding!("loopIteration", FieldPredictor::Inc, FieldEncoding::Null);
        assert_p_field_encoding!("time", FieldPredictor::StraightLine, FieldEncoding::SignedVb);

        let mut blackbox = Logger::default();
        blackbox.main_data[0].time_us = 3;
        blackbox.main_data[1].time_us = 2;
        blackbox.main_data[2].time_us = 1;
        blackbox.main_data[0].gyro[0] = 1000;

        let mut buffer = [0u8; 512];
        let mut encoder = SliceEncoder { buffer: &mut buffer, pos: 0 };

        // Ensures the assert_p_field_encoding macros are run.
        let p_frame_start_pos = blackbox.log_p_frame(&mut encoder);
        assert_eq!(0, p_frame_start_pos);
        assert_eq!(b'P', encoder.buffer[p_frame_start_pos]);
        assert_eq!(0, encoder.buffer[p_frame_start_pos + 1]);

        assert_eq!(1, blackbox.main_data[blackbox.main_data_current_idx].time_us);
        assert_eq!(3, blackbox.main_data[blackbox.main_data_previous_idx].time_us);
        assert_eq!(2, blackbox.main_data[blackbox.main_data_pre_previous_idx].time_us);
        assert_eq!(1000, blackbox.main_data[blackbox.main_data_previous_idx].gyro[0]);
        assert_eq!(0, blackbox.main_data[blackbox.main_data_pre_previous_idx].gyro[0]);

        blackbox.main_data[0].time_us = 4;
        let p_frame_start_pos = blackbox.log_p_frame(&mut encoder);
        assert_eq!(2, p_frame_start_pos);
        assert_eq!(b'P', encoder.buffer[p_frame_start_pos]);
        assert_eq!(9, encoder.buffer[p_frame_start_pos + 1]);

        assert_eq!(2, blackbox.main_data[blackbox.main_data_current_idx].time_us);
        assert_eq!(1, blackbox.main_data[blackbox.main_data_previous_idx].time_us);
        assert_eq!(4, blackbox.main_data[blackbox.main_data_pre_previous_idx].time_us);
        assert_eq!(1000, blackbox.main_data[blackbox.main_data_pre_previous_idx].gyro[0]);

        blackbox.main_data[0].time_us = 5;
        let p_frame_start_pos = blackbox.log_p_frame(&mut encoder);
        assert_eq!(4, p_frame_start_pos);
        assert_eq!(b'P', encoder.buffer[p_frame_start_pos]);
        assert_eq!(10, encoder.buffer[p_frame_start_pos + 1]);

        assert_eq!(5, blackbox.main_data[blackbox.main_data_current_idx].time_us);
        assert_eq!(2, blackbox.main_data[blackbox.main_data_previous_idx].time_us);
        assert_eq!(1, blackbox.main_data[blackbox.main_data_pre_previous_idx].time_us);
    }

    #[cfg(feature = "huffman")]
    #[test]
    fn q_encodings() {
        let mut blackbox = Logger::default();
        let mut buffer = [0u8; 32];
        let mut encoder = SliceEncoder { buffer: &mut buffer, pos: 0 };

        // Simulate encoding a p_frame into the encoder buffer.
        let p_frame = [b'P', 0, 1, 2, 3, 4];
        let p_frame_start_pos = encoder.pos;
        encoder.buffer[0..6].copy_from_slice(&p_frame);
        encoder.pos += p_frame.len();
        assert_eq!(6, encoder.pos);

        let p_frame_length = encoder.pos - p_frame_start_pos;
        assert_eq!(b'P', encoder.buffer[p_frame_start_pos]);
        assert_eq!(0, encoder.buffer[p_frame_start_pos + p_frame_length]);
        assert_eq!(0, p_frame_start_pos);
        assert_eq!(6, p_frame_length);

        let result = blackbox.try_convert_p_frame_to_q_frame(&mut encoder, p_frame_start_pos);
        assert!(result);
        assert_eq!(5, encoder.pos); // compressed length is 5 bytes, which is less than the original p_frame length of 6 bytes.
        assert_eq!(b'Q', encoder.buffer[0]);
        assert_eq!(p_frame_length - 1, encoder.buffer[1] as usize); // uncompressed input length, does not include the 'P' character.
        assert_eq!(0xD9, encoder.buffer[2]);
        assert_eq!(0xB8, encoder.buffer[3]);
        assert_eq!(0x80, encoder.buffer[4]);
        assert_eq!(4, encoder.buffer[5]); // left over from the original p_frame
        assert_eq!(0, encoder.buffer[6]);
        assert_eq!(&[b'Q', 5, 0xD9, 0xB8, 0x80, 4, 0], &encoder.buffer[0..=6]);

        // Now simulate encoding another p_frame.
        let p_frame_start_pos = encoder.pos;
        encoder.buffer[5..11].copy_from_slice(&p_frame);
        encoder.pos += p_frame.len();
        assert_eq!(11, encoder.pos);

        let p_frame_length = encoder.pos - p_frame_start_pos;
        assert_eq!(b'P', encoder.buffer[5]);
        //assert_eq!(0, encoder.buffer[p_frame_start_pos + p_frame_length]);
        assert_eq!(5, p_frame_start_pos);
        assert_eq!(6, p_frame_length);

        let _result = blackbox.try_convert_p_frame_to_q_frame(&mut encoder, p_frame_start_pos);
        assert!(result);
        assert_eq!(10, encoder.pos); // compressed length is 5 bytes, which is less than the original p_frame length of 6 bytes.
        assert_eq!(b'Q', encoder.buffer[5]);
        assert_eq!(p_frame_length - 1, encoder.buffer[6] as usize); // uncompressed input length, does not include the 'P' character.
        assert_eq!(0xD9, encoder.buffer[7]);
        assert_eq!(0xB8, encoder.buffer[8]);
        assert_eq!(0x80, encoder.buffer[9]);
        assert_eq!(4, encoder.buffer[10]); // left over from the original p_frame
        assert_eq!(0, encoder.buffer[11]);
        assert_eq!(&[b'Q', 5, 0xD9, 0xB8, 0x80, 4, 0], &encoder.buffer[5..=11]);

        // Now simulate encoding a p_frame that is larger when compressed.
        let p_frame_larger_when_compressed = [b'P', 0x80, 0x81, 0x82, 0x83, 0x84];
        let p_frame_start_pos = encoder.pos;
        encoder.buffer[10..16].copy_from_slice(&p_frame_larger_when_compressed);
        encoder.pos += p_frame.len();
        assert_eq!(16, encoder.pos);

        let p_frame_length = encoder.pos - p_frame_start_pos;
        assert_eq!(b'P', encoder.buffer[10]);
        assert_eq!(0, encoder.buffer[p_frame_start_pos + p_frame_length]);
        assert_eq!(10, p_frame_start_pos);
        assert_eq!(6, p_frame_length);

        let result = blackbox.try_convert_p_frame_to_q_frame(&mut encoder, p_frame_start_pos);
        assert!(!result);
        assert_eq!(16, encoder.pos);
        assert_eq!(b'P', encoder.buffer[10]);
        assert_eq!(0x80, encoder.buffer[11]);
        assert_eq!(0x81, encoder.buffer[12]);
        assert_eq!(0x82, encoder.buffer[13]);
        assert_eq!(0x83, encoder.buffer[14]);
        assert_eq!(0x84, encoder.buffer[15]);
        assert_eq!(&p_frame_larger_when_compressed, &encoder.buffer[10..=15]);
    }
}
