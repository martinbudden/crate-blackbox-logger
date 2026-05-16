use crate::data::{Event, EventId, MainData};
use crate::field_definitions::{FieldCondition, FieldSelect};
use crate::logger::Logger;
use crate::{BlackboxWriter, SliceWriter};

#[cfg(test)]
use crate::field_definitions::{FieldEncoding, FieldPredictor, MainFieldDefinition};

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
    /// Log event: e_frame. Written immediately to log when event occurs.
    pub fn log_e_frame(&mut self, encoder: &mut SliceWriter, event: Event) -> usize {
        encoder.begin_frame(b'E');

        match event {
            Event::SyncBeep(time) => {
                encoder.write_byte(EventId::SYNC_BEEP);
                encoder.write_unsigned_vb(time);
            }
            Event::InflightAdjustment(new_value, new_value_f32, adjustment, is_float) => {
                encoder.write_byte(EventId::INFLIGHT_ADJUSTMENT);
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
            Event::Disarm(reason) => {
                encoder.write_byte(EventId::DISARM);
                encoder.write_unsigned_vb(reason);
            }
            Event::LoggingResume(iteration, time) => {
                encoder.write_byte(EventId::LOGGING_RESUME);
                encoder.write_unsigned_vb(iteration);
                encoder.write_unsigned_vb(time);
            }
            Event::FlightMode(flags, previous_flags) => {
                encoder.write_byte(EventId::FLIGHT_MODE);
                encoder.write_unsigned_vb(flags);
                encoder.write_unsigned_vb(previous_flags);
            }
            Event::LogEnd => {
                encoder.write_byte(EventId::LOG_END);
                encoder.write_str("End of log");
                encoder.write_byte(0);
                // TODO:
            }
            _ => {}
        }
        encoder.end_frame()
    }

    /// Log slow frame: s_frame.
    pub fn log_s_frame(&mut self, encoder: &mut SliceWriter) -> usize {
        self.logged_any_frames = true;
        self.s_frame_index = 0;
        self.new_slow_data = false;

        encoder.begin_frame(b'S');

        encoder.write_unsigned_vb(self.slow_data.flight_mode_flags);
        encoder.write_unsigned_vb(u32::from(self.slow_data.state_flags));

        // Most of the time these three values will be able to pack into one byte.
        let values = [
            i32::from(self.slow_data.failsafe_phase),
            i32::from(self.slow_data.rx_signal_received),
            i32::from(self.slow_data.rx_flight_channel_is_valid),
        ];

        encoder.write_tag2_3s32(values);

        encoder.end_frame()
    }

    /// GPS home frame: h_frame.
    pub fn log_h_frame(&mut self, encoder: &mut SliceWriter) -> usize {
        self.logged_any_frames = true;

        encoder.begin_frame(b'H');

        encoder.write_signed_vb(self.gps_data.home.latitude_degrees_1e7);
        encoder.write_signed_vb(self.gps_data.home.longitude_degrees_1e7);
        // log altitude in increments of 0.1m
        encoder.write_signed_vb(self.gps_data.home.altitude_cm / 10);

        encoder.end_frame()
    }

    /// GPS frame: g_frame. Written at a frequency of about 10Hz.
    pub fn log_g_frame(&mut self, current_time_us: u32, encoder: &mut SliceWriter) -> usize {
        self.logged_any_frames = true;
        self.new_gps_data = false;

        encoder.begin_frame(b'G');

        // If we're logging every frame, then a GPS frame always appears just after a frame with the
        // current_time timestamp in the log, so the reader can just use that timestamp for the GPS frame.
        // If we're not logging every frame, we need to store the time of this GPS frame.
        if self.conditions.test(FieldCondition::NOT_LOGGING_EVERY_FRAME) {
            // Predict the time of the last frame in the main log
            encoder.write_unsigned_vb(current_time_us - self.main_data[0].time_us);
        }

        encoder.write_unsigned_vb(u32::from(self.gps_data.satellite_count));
        encoder.write_signed_vb(self.gps_data.position.latitude_degrees_1e7 - self.gps_home.latitude_degrees_1e7);
        encoder.write_signed_vb(self.gps_data.position.longitude_degrees_1e7 - self.gps_home.longitude_degrees_1e7);
        // log altitude in increments of 0.1m
        encoder.write_signed_vb(self.gps_data.position.altitude_cm / 10);

        #[allow(clippy::cast_sign_loss)]
        //if self.config.gps_use_3d_speed {
        //    encoder.write_unsigned_vb(self.gps_data.speed3d_cmps as u32);
        //} else {
        encoder.write_unsigned_vb(self.gps_data.ground_speed_cmps as u32);
        //}

        #[allow(clippy::cast_sign_loss)]
        encoder.write_unsigned_vb(self.gps_data.ground_course_deci_degrees as u32);

        encoder.write_signed_vb_16(self.gps_data.velocity_north_cmps);
        encoder.write_signed_vb_16(self.gps_data.velocity_east_cmps);
        encoder.write_signed_vb_16(self.gps_data.velocity_down_cmps);

        encoder.end_frame()
    }

    /// Write an Intra frame (i_frame).
    /// Also known as a key frame.
    #[allow(clippy::too_many_lines)]
    pub fn log_i_frame(&mut self, encoder: &mut SliceWriter) -> usize {
        self.logged_any_frames = true;
        let current = &self.main_data[0];

        encoder.begin_frame(b'I');

        assert_i_field_encoding!("loopIteration", FieldPredictor::ZERO, FieldEncoding::UNSIGNED_VB);
        encoder.write_unsigned_vb(self.iteration);

        assert_i_field_encoding!("time", FieldPredictor::ZERO, FieldEncoding::UNSIGNED_VB);
        encoder.write_unsigned_vb(current.time_us);

        assert_i_field_encoding!("axisP", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
        assert_i_field_encoding!("axisI", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
        assert_i_field_encoding!("axisD", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
        assert_i_field_encoding!("axisF", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
        assert_i_field_encoding!("axisS", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
        if self.conditions.test(FieldCondition::PID) {
            encoder.write_signed_vb_array(&current.pid_p);
            assert_i_field_encoding!("axisI", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
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
        assert_i_field_encoding!("rcCommand", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
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
            encoder.write_unsigned_vb(u32::from(current.rc_commands[3].wrapping_sub(self.min_throttle)));
        }

        assert_i_field_encoding!("setpoint", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
        if self.conditions.test(FieldCondition::SETPOINT) {
            // Write setpoint roll, pitch, yaw, and throttle
            encoder.write_signed_vb_16_array(&current.setpoints);
        }

        assert_i_field_encoding!("vbatLatest", FieldPredictor::VBATREF, FieldEncoding::NEG_14BIT);
        if self.conditions.test(FieldCondition::BATTERY_VOLTAGE) {
            // Our voltage is expected to decrease over the course of the flight, so store our difference from the reference.
            // Write 14 bits even if the number is negative (which would otherwise result in 32 bits)
            encoder.write_unsigned_vb(u32::from(self.vbat_reference - current.battery_voltage) & 0x3FFF);
        }

        assert_i_field_encoding!("amperageLatest", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
        if self.conditions.test(FieldCondition::BATTERY_CURRENT) {
            // 12bit value directly from ADC
            encoder.write_signed_vb_16(current.amperage);
        }

        assert_i_field_encoding!("BaroAlt", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
        if self.conditions.test(FieldCondition::BAROMETER) {
            encoder.write_signed_vb(current.baro_altitude);
        }

        assert_i_field_encoding!("surfaceRaw", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
        if self.conditions.test(FieldCondition::RANGEFINDER) {
            encoder.write_signed_vb(current.range_raw);
        }

        assert_i_field_encoding!("rssi", FieldPredictor::ZERO, FieldEncoding::UNSIGNED_VB);
        if self.conditions.test(FieldCondition::RSSI) {
            encoder.write_unsigned_vb_16(current.rssi);
        }

        assert_i_field_encoding!("magADC", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
        if self.conditions.test(FieldCondition::MAGNETOMETER) {
            encoder.write_signed_vb_16_array(&current.mag);
        }

        assert_i_field_encoding!("gyroADC", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
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

        assert_i_field_encoding!("motor", FieldPredictor::MIN_MOTOR, FieldEncoding::SIGNED_VB);
        if Logger::field_enabled(self.enabled_fields, FieldSelect::MOTOR) {
            // Motors can be below minimum output when disarmed, but that doesn't happen much
            encoder.write_signed_vb_16(current.motor[0].wrapping_sub(self.min_throttle).cast_signed());

            // Motors tend to be similar to each other so use the first motor's value as a predicted of the others
            for ii in 1..self.motor_count {
                encoder.write_signed_vb_16(current.motor[ii].wrapping_sub(current.motor[0]).cast_signed());
            }
        }
        #[cfg(feature = "dshot_telemetry")]
        assert_i_field_encoding!("eRPM", FieldPredictor::ZERO, FieldEncoding::UNSIGNED_VB);
        #[cfg(feature = "dshot_telemetry")]
        if Logger::field_enabled(self.enabled_fields, FieldSelect::MOTOR_RPM) {
            for erpm in current.erpm {
                encoder.write_unsigned_vb_16(erpm);
            }
        }
        #[cfg(feature = "servos")]
        if self.conditions.test(FieldCondition::SERVOS) {
            let out: [i32; MainData::MAX_SUPPORTED_SERVO_COUNT] = std::array::from_fn(|i| {
                i32::from(current.servos[i]) - crate::field_definitions::FieldPredictor::S_1500
            });
            encoder.write_tag8_8svb(&out);
        }

        let ret = encoder.end_frame();

        // This is an i_frame, so there is no other previous data, so we copy the current data into the pre_previous data.
        self.main_data[2] = self.main_data[0];
        self.main_data[1] = self.main_data[0];

        ret
    }

    /// Write a Predictor frame (p_frame).
    /// Also known as an inter frame.
    /// Note: the predictions are hard coded to match the values defined in BLACKBOX_MAIN_FIELDS:
    /// the code is made safe by asserting the p_encoding values.
    /// So this code and those definitions must be changed in tandem with each other.
    #[allow(clippy::too_many_lines)]
    pub fn log_p_frame(&mut self, encoder: &mut SliceWriter) -> usize {
        self.logged_any_frames = true;

        {
            let current = &self.main_data[0];
            let previous = &self.main_data[1];
            let pre_previous = &self.main_data[2];

            encoder.begin_frame(b'P');

            // Don't store store iteration when using FieldEncoding::NULL
            assert_p_field_encoding!("loopIteration", FieldPredictor::INC, FieldEncoding::NULL);

            // Since the difference between the difference between successive times will be nearly zero (due to consistent
            // loop time spacing), use second-order differences.
            assert_p_field_encoding!("time", FieldPredictor::STRAIGHT_LINE, FieldEncoding::SIGNED_VB);
            let time: i64 =
                i64::from(current.time_us) - 2 * i64::from(previous.time_us) + i64::from(pre_previous.time_us);
            #[allow(clippy::cast_possible_truncation)]
            encoder.write_signed_vb(time as i32);

            assert_p_field_encoding!("axisP", FieldPredictor::PREVIOUS, FieldEncoding::SIGNED_VB);
            assert_p_field_encoding!("axisI", FieldPredictor::PREVIOUS, FieldEncoding::TAG2_3S32);
            assert_p_field_encoding!("axisD", FieldPredictor::PREVIOUS, FieldEncoding::SIGNED_VB);
            assert_p_field_encoding!("axisF", FieldPredictor::PREVIOUS, FieldEncoding::SIGNED_VB);
            assert_p_field_encoding!("axisS", FieldPredictor::PREVIOUS, FieldEncoding::SIGNED_VB);
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

            // RC tends to stay the same or fairly small for many frames at a time, so use an encoding that
            assert_p_field_encoding!("rcCommand", FieldPredictor::PREVIOUS, FieldEncoding::TAG8_4S16);
            if self.conditions.test(FieldCondition::RC_COMMANDS) {
                let deltas = [
                    current.rc_commands[0].wrapping_sub(previous.rc_commands[0]).cast_signed(),
                    current.rc_commands[1].wrapping_sub(previous.rc_commands[1]).cast_signed(),
                    current.rc_commands[2].wrapping_sub(previous.rc_commands[2]).cast_signed(),
                    current.rc_commands[3].wrapping_sub(previous.rc_commands[3]).cast_signed(),
                ];
                encoder.write_tag8_4s16(deltas);
            }
            assert_p_field_encoding!("setpoint", FieldPredictor::PREVIOUS, FieldEncoding::TAG8_4S16);
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
            let mut deltas = <[i32; 8]>::default();
            let mut tag8_field_count = 0_usize;

            assert_p_field_encoding!("vbatLatest", FieldPredictor::PREVIOUS, FieldEncoding::TAG8_8SVB);
            if self.conditions.test(FieldCondition::BATTERY_VOLTAGE) {
                deltas[tag8_field_count] = i32::from(current.battery_voltage.wrapping_sub(previous.battery_voltage));
                tag8_field_count += 1;
            }
            assert_p_field_encoding!("amperageLatest", FieldPredictor::PREVIOUS, FieldEncoding::TAG8_8SVB);
            if self.conditions.test(FieldCondition::BATTERY_CURRENT) {
                deltas[tag8_field_count] = i32::from(current.amperage.wrapping_sub(previous.amperage));
                tag8_field_count += 1;
            }
            assert_p_field_encoding!("BaroAlt", FieldPredictor::PREVIOUS, FieldEncoding::TAG8_8SVB);
            if self.conditions.test(FieldCondition::BAROMETER) {
                deltas[tag8_field_count] = current.baro_altitude.wrapping_sub(previous.baro_altitude);
                tag8_field_count += 1;
            }
            assert_p_field_encoding!("surfaceRaw", FieldPredictor::PREVIOUS, FieldEncoding::TAG8_8SVB);
            if self.conditions.test(FieldCondition::RANGEFINDER) {
                deltas[tag8_field_count] = current.range_raw.wrapping_sub(previous.range_raw);
                tag8_field_count += 1;
            }
            assert_p_field_encoding!("rssi", FieldPredictor::PREVIOUS, FieldEncoding::TAG8_8SVB);
            if self.conditions.test(FieldCondition::RSSI) {
                deltas[tag8_field_count] = i32::from(current.rssi.wrapping_sub(previous.rssi));
                tag8_field_count += 1;
            }
            assert_p_field_encoding!("magADC", FieldPredictor::PREVIOUS, FieldEncoding::TAG8_8SVB);
            if self.conditions.test(FieldCondition::MAGNETOMETER) {
                for ii in 0..MainData::XYZ_AXIS_COUNT {
                    deltas[tag8_field_count] = i32::from(current.mag[ii].wrapping_sub(previous.mag[ii]));
                    tag8_field_count += 1;
                }
            }

            if tag8_field_count > 0 {
                encoder.write_tag8_8svb(&deltas);
            }

            // Since gyros, accelerometers and motors are noisy, base their predictions on the average of the history:
            assert_p_field_encoding!("gyroADC", FieldPredictor::PREVIOUS, FieldEncoding::SIGNED_VB);
            if self.conditions.test(FieldCondition::GYRO) {
                for ii in 0..MainData::XYZ_AXIS_COUNT {
                    encoder.write_signed_vb_16(current.gyro[ii] - previous.gyro[ii]);
                }
            }
            assert_p_field_encoding!("gyroUnfilt", FieldPredictor::AVERAGE_2, FieldEncoding::SIGNED_VB);
            if self.conditions.test(FieldCondition::GYRO_UNFILTERED) {
                for ii in 0..MainData::XYZ_AXIS_COUNT {
                    let predicted = i16::midpoint(previous.gyro_unfiltered[ii], pre_previous.gyro_unfiltered[ii]);
                    encoder.write_signed_vb_16(current.gyro_unfiltered[ii].wrapping_sub(predicted));
                }
            }
            assert_p_field_encoding!("accSmooth", FieldPredictor::AVERAGE_2, FieldEncoding::SIGNED_VB);
            if self.conditions.test(FieldCondition::ACC) {
                for ii in 0..MainData::XYZ_AXIS_COUNT {
                    let predicted = i16::midpoint(previous.acc[ii], pre_previous.acc[ii]);
                    encoder.write_signed_vb_16(current.acc[ii].wrapping_sub(predicted));
                }
            }
            assert_p_field_encoding!("imuQuaternion", FieldPredictor::AVERAGE_2, FieldEncoding::SIGNED_VB);
            if self.conditions.test(FieldCondition::ATTITUDE) {
                for ii in 0..MainData::XYZ_AXIS_COUNT {
                    let predicted = i16::midpoint(previous.orientation[ii], pre_previous.orientation[ii]);
                    encoder.write_signed_vb_16(current.orientation[ii].wrapping_sub(predicted));
                }
            }

            assert_p_field_encoding!("debug", FieldPredictor::AVERAGE_2, FieldEncoding::SIGNED_VB);
            if self.conditions.test(FieldCondition::DEBUG) {
                for ii in 0..MainData::DEBUG_COUNT {
                    let predicted = i16::midpoint(previous.debug[ii], pre_previous.debug[ii]);
                    encoder.write_signed_vb_16(current.debug[ii].wrapping_sub(predicted));
                }
            }
            assert_p_field_encoding!("motor", FieldPredictor::AVERAGE_2, FieldEncoding::SIGNED_VB);
            if Logger::field_enabled(self.enabled_fields, FieldSelect::MOTOR) {
                for ii in 0..self.motor_count {
                    let predicted = u16::midpoint(previous.motor[ii], pre_previous.motor[ii]);
                    encoder.write_signed_vb_16(current.motor[ii].wrapping_sub(predicted).cast_signed());
                }
            }
            #[cfg(feature = "dshot_telemetry")]
            assert_p_field_encoding!("eRPM", FieldPredictor::PREVIOUS, FieldEncoding::SIGNED_VB);
            #[cfg(feature = "dshot_telemetry")]
            if Logger::field_enabled(self.enabled_fields, FieldSelect::MOTOR_RPM) {
                for ii in 0..self.motor_count {
                    encoder.write_signed_vb_16(current.erpm[ii].wrapping_sub(previous.erpm[ii]).cast_signed());
                }
            }

            #[cfg(feature = "servos")]
            if self.conditions.test(FieldCondition::SERVOS) {
                let servos: [i32; MainData::MAX_SUPPORTED_SERVO_COUNT] = core::array::from_fn(|ii| {
                    i32::from(current.servos[ii]) - crate::field_definitions::FieldPredictor::S_1500
                });
                encoder.write_tag8_8svb(&servos);
            }
        }
        let ret = encoder.end_frame();

        // Rotate the saved data.
        self.main_data[2] = self.main_data[1];
        self.main_data[1] = self.main_data[0];

        ret
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn i_encodings() {
        assert_i_field_encoding!("loopIteration", FieldPredictor::ZERO, FieldEncoding::UNSIGNED_VB);
        assert_i_field_encoding!("time", FieldPredictor::ZERO, FieldEncoding::UNSIGNED_VB);

        let mut blackbox = Logger::default();
        blackbox.main_data[0].time_us = 3;
        blackbox.main_data[1].time_us = 2;
        blackbox.main_data[2].time_us = 1;

        let mut buffer = [0u8; 512];
        let mut encoder = SliceWriter { buffer: &mut buffer, pos: 0 };

        _ = blackbox.log_i_frame(&mut encoder);

        assert_eq!(3, blackbox.main_data[0].time_us);
        assert_eq!(3, blackbox.main_data[1].time_us);
        assert_eq!(3, blackbox.main_data[2].time_us);

        blackbox.main_data[0].time_us = 4;
        _ = blackbox.log_i_frame(&mut encoder);
        assert_eq!(4, blackbox.main_data[0].time_us);
        assert_eq!(4, blackbox.main_data[1].time_us);
        assert_eq!(4, blackbox.main_data[2].time_us);
    }
    #[test]
    fn p_encodings() {
        assert_p_field_encoding!("loopIteration", FieldPredictor::INC, FieldEncoding::NULL);
        assert_p_field_encoding!("time", FieldPredictor::STRAIGHT_LINE, FieldEncoding::SIGNED_VB);

        let mut blackbox = Logger::default();
        blackbox.main_data[0].time_us = 3;
        blackbox.main_data[1].time_us = 2;
        blackbox.main_data[2].time_us = 1;
        blackbox.main_data[0].gyro[0] = 1000;

        let mut buffer = [0u8; 512];
        let mut encoder = SliceWriter { buffer: &mut buffer, pos: 0 };

        _ = blackbox.log_p_frame(&mut encoder);
        assert_eq!(3, blackbox.main_data[0].time_us);
        assert_eq!(3, blackbox.main_data[1].time_us);
        assert_eq!(2, blackbox.main_data[2].time_us);
        assert_eq!(1000, blackbox.main_data[1].gyro[0]);
        assert_eq!(0, blackbox.main_data[2].gyro[0]);

        blackbox.main_data[0].time_us = 4;
        _ = blackbox.log_p_frame(&mut encoder);
        assert_eq!(4, blackbox.main_data[0].time_us);
        assert_eq!(4, blackbox.main_data[1].time_us);
        assert_eq!(3, blackbox.main_data[2].time_us);
        assert_eq!(1000, blackbox.main_data[2].gyro[0]);

        blackbox.main_data[0].time_us = 5;
        _ = blackbox.log_p_frame(&mut encoder);
        assert_eq!(5, blackbox.main_data[0].time_us);
        assert_eq!(5, blackbox.main_data[1].time_us);
        assert_eq!(4, blackbox.main_data[2].time_us);
    }
}
