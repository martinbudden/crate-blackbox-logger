use crate::BlackboxCallbacks;
use crate::{FieldCondition, LogFieldSelect, SliceWriter};
#[cfg(test)]
use crate::{FieldEncoding, FieldPredictor, MainFieldDefinition};
use crate::{GpsState, MainState, SlowState};
use receivers::BitSet64;
use serde::{Deserialize, Serialize};

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

pub struct BlackboxDevice {}
impl BlackboxDevice {
    pub const NONE: u8 = 0;
    pub const FLASH: u8 = 1;
    pub const SDCARD: u8 = 2;
    pub const SERIAL: u8 = 3;
}

pub struct BlackboxMode {}
impl BlackboxMode {
    pub const NORMAL: u8 = 0;
    pub const MOTOR_TEST: u8 = 1;
    pub const ALWAYS_ON: u8 = 2;
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct BlackboxConfig {
    pub sample_rate: u8,
    pub device: u8,
    pub mode: u8,
    pub gps_use_3d_speed: bool,
    pub fields_disabled_mask: u32,
}

impl Default for BlackboxConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl BlackboxConfig {
    fn new() -> Self {
        Self {
            sample_rate: 0,
            device: BlackboxDevice::NONE,
            mode: BlackboxMode::NORMAL,
            gps_use_3d_speed: false,
            fields_disabled_mask: 0,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlackboxStart {
    pub debug_mode: u16,
    pub motor_count: u8,
    pub servo_count: u8,
}

impl Default for BlackboxStart {
    fn default() -> Self {
        Self::new()
    }
}

impl BlackboxStart {
    fn new() -> Self {
        Self { debug_mode: 0, motor_count: 4, servo_count: 0 }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Blackbox {
    iteration: u32,
    loop_index: u32,

    motor_count: usize,
    servo_count: usize,
    debug_mode: u32,
    motor_output_min: i16,
    min_throttle: i16,
    vbat_reference: u16,
    logged_any_frames: bool,

    i_frame_index: u32,
    i_interval: u32,
    p_frame_index: u32,
    p_interval: u32,
    s_frame_index: u32,
    s_interval: u32,

    condition_cache: BitSet64,
    log_select_enabled: u32,

    slow_state: SlowState,
    gps_state: GpsState,
    home_longitude_degrees_1e7: i32, // home longitude in degrees * 1e+7
    home_latitude_degrees_1e7: i32,  // home latitude in degrees * 1e+7
    home_altitude_cm: i32,           // home altitude in cm

    main_states: [MainState; 3],
    state_index_current: usize,
    state_index_previous: usize,
    state_index_pre_previous: usize,
    config: BlackboxConfig,
    buf: [u8; 1024],
    /// Callbacks. Probably better done with Generics, but this is simpler for now.
    callbacks: BlackboxCallbacks,
}

impl Default for Blackbox {
    fn default() -> Self {
        Self::new(BlackboxCallbacks::default())
    }
}

impl Blackbox {
    pub fn new(callbacks: BlackboxCallbacks) -> Self {
        Self {
            iteration: 0,
            loop_index: 0,
            motor_count: 4,
            servo_count: 0,
            debug_mode: 0,
            motor_output_min: 750,
            min_throttle: 700,
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
            home_longitude_degrees_1e7: 0,
            home_latitude_degrees_1e7: 0,
            home_altitude_cm: 0,
            main_states: <[MainState; 3]>::default(),
            state_index_current: 0,
            state_index_previous: 1,
            state_index_pre_previous: 2,
            config: BlackboxConfig::default(),
            buf: [0u8; 1024],
            callbacks,
        }
    }
}

impl Blackbox {
    pub fn init(&mut self, config: BlackboxConfig) {
        //_serial_device.init();

        self.config = config;

        self.log_select_enabled = LogFieldSelect::PID
        | LogFieldSelect::PID_KTERM
        | LogFieldSelect::PID_DTERM_ROLL
        | LogFieldSelect::PID_DTERM_PITCH
        //| LogFieldSelect::PID_STERM_ROLL
        //| LogFieldSelect::PID_STERM_PITCH
        //| LogFieldSelect::PID_STERM_YAW
        | LogFieldSelect::SETPOINT
        | LogFieldSelect::RC_COMMANDS
        | LogFieldSelect::GYRO
        | LogFieldSelect::GYRO_UNFILTERED
        | LogFieldSelect::ACCELEROMETER
        //| LogFieldSelect::ATTITUDE
        | LogFieldSelect::MOTOR
        | LogFieldSelect::MOTOR_RPM;

        self.reset_iteration_timers();

        // an i_frame is written every 32ms
        // blackboxUpdate() is run in synchronization with the PID loop
        // target_pid_looptime_us is 1000 for 1kHz loop, 500 for 2kHz loop etc, target_pid_looptime_us is rounded for short looptimes
        // TODO: self.i_interval = 32 * 1000 / self.target_pid_looptime_us;

        self.p_interval = 1 << config.sample_rate;
        if self.p_interval > self.i_interval {
            self.p_interval = 0; // log only i_frames if logging frequency is too low
        }

        // S-frame is written every 256*32 = 8192ms, approx every 8 seconds
        self.s_interval = self.i_interval * 256;

        /*if config.device == BlackboxDevice::NONE {
            self.set_state(STATE_DISABLED);
        } else if (config.mode == BlackboxMode::ALWAYS_ON) {
            self.start();
        } else {
            self.set_state(STATE_STOPPED);
        }*/
    }
    pub fn start(&self, _start_params: BlackboxStart) {}
    pub fn finish(&self) {}

    /// Build condition cache, called from start().
    pub fn build_field_condition_cache(&mut self) {
        self.condition_cache.reset_all();
        for condition in FieldCondition::FIRST..FieldCondition::LAST {
            if self.test_field_condition_uncached(condition) {
                _ = self.condition_cache.set(condition);
            }
        }
    }

    pub fn reset_iteration_timers(&mut self) {
        self.iteration = 0;
        self.loop_index = 0;
        self.i_frame_index = 0;
        self.p_frame_index = 0;
        self.s_frame_index = 0;
    }
    /// Called once every FC loop in order to keep track of how many FC loop iterations have passed.
    pub fn advance_iteration_timers(&mut self) {
        self.s_frame_index += 1;
        self.iteration += 1;
        self.loop_index += 1;

        if self.loop_index >= self.i_interval {
            self.loop_index = 0; // value of zero means i_frame will be written on next update
            self.i_frame_index += 1;
            self.p_frame_index = 0;
        } else {
            self.p_frame_index += 1;
            if self.p_frame_index >= self.p_interval {
                self.p_frame_index = 0; // value of zero means p_frame will be written on next update, if i_frame not written
            }
        }
    }

    /// Called when the flight controller signals it has new data.
    pub fn log_iteration(&mut self, current_time_us: u32) {
        // Write a keyframe every i_interval frames so we can resynchronise upon missing frames
        if self.should_log_i_frame() {
            // ie _loop_index == 0
            // Don't log a slow frame if the slow data didn't change (i_frames are already large enough without adding
            // an additional item to write at the same time). Unless we're *only* logging i_frames, then we have no choice.
            if self.is_only_logging_i_frames() {
                let _len = self.log_s_frame_if_needed();
                //self.sd_card.write_all(&self.buf[..len]).await.ok();
            }

            (self.callbacks.load_main_state)(&mut self.main_states[self.state_index_current], current_time_us);
            let _len = self.log_i_frame();
            //self.sd_card.write_all(&self.buf[..len]).await.ok();
        } else {
            self.log_event_arming_beep_if_needed();
            self.log_event_flight_mode_if_needed(); // Check for FlightMode status change event

            if self.should_log_p_frame() {
                // ie p_frame_index == 0 && p_interval != 0
                // We assume that slow frames are only interesting in that they aid the interpretation of the main data stream.
                // So only log slow frames during loop iterations where we log a main frame.
                let _len = self.log_s_frame_if_needed();
                //self.sd_card.write_all(&self.buf[..len]).await.ok();

                (self.callbacks.load_main_state)(&mut self.main_states[self.state_index_current], current_time_us);
                let _len = self.log_p_frame();
                //self.sd_card.write_all(&self.buf[..len]).await.ok();
            }
            #[cfg(feature = "gps")]
            if Self::field_enabled(self.log_select_enabled, LogFieldSelect::GPS) {
                let mut gps_state_new = GpsState::new();
                (self.callbacks.load_gps_state)(&mut gps_state_new);

                let gps_state_changed = gps_state_new.satellite_count != self.gps_state.satellite_count
                    || gps_state_new.latitude_degrees_1e7 != self.gps_state.latitude_degrees_1e7
                    || gps_state_new.longitude_degrees_1e7 != self.gps_state.longitude_degrees_1e7;

                self.gps_state = gps_state_new;

                if self.should_log_h_frame() {
                    self.home_latitude_degrees_1e7 = self.gps_state.home_latitude_degrees_1e7;
                    self.home_longitude_degrees_1e7 = self.gps_state.home_longitude_degrees_1e7;
                    self.home_altitude_cm = self.gps_state.home_altitude_cm;
                    let _len = self.log_h_frame();
                    //self.sd_card.write_all(&self.buf[..len]).await.ok();
                    let _len = self.log_g_frame(current_time_us);
                //self.sd_card.write_all(&self.buf[..len]).await.ok();
                } else if gps_state_changed {
                    //We could check for velocity changes as well but I doubt it changes independent of position
                    let _len = self.log_g_frame(current_time_us);
                    //self.sd_card.write_all(&self.buf[..len]).await.ok();
                }
            }
        }
    }

    pub fn should_log_i_frame(&self) -> bool {
        self.loop_index == 0
    }
    pub fn should_log_h_frame(&self) -> bool {
        true
    }
    pub fn should_log_p_frame(&self) -> bool {
        self.p_frame_index == 0 && self.p_interval != 0
    }
    pub fn is_only_logging_i_frames(&self) -> bool {
        self.p_interval == 0
    }

    pub fn log_event_arming_beep_if_needed(&self) {}
    pub fn log_event_flight_mode_if_needed(&self) {} // Check for FlightMode status change event

    /// If the data in the slow frame has changed, log a slow frame.
    ///
    /// The frame is also logged if it has been more than s_interval logging iterations
    /// since the field was last logged.
    pub fn log_s_frame_if_needed(&mut self) -> usize {
        // Write the slow frame periodically so it can be recovered if we ever lose sync
        if self.s_frame_index >= self.s_interval {
            (self.callbacks.load_slow_state)(&mut self.slow_state);
            return self.log_s_frame();
        }

        // Only write a slow frame if it was different from the previous state
        let mut new_slow_state = SlowState::new();
        (self.callbacks.load_slow_state)(&mut new_slow_state);
        if new_slow_state != self.slow_state {
            // Use the new state as our new history
            self.slow_state = new_slow_state;
            return self.log_s_frame();
        }
        0
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

/// Write the contents of slow_state to the log as an s_frame.
/// Returns the length written.
impl Blackbox {
    pub fn log_s_frame(&mut self) -> usize {
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

        encoder.end_frame()
    }

    pub fn log_s_frame2(&mut self, encoder: &mut SliceWriter) -> usize {
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

        encoder.end_frame()
    }
}

impl Blackbox {
    /// GPS home frame: h_frame.
    pub fn log_h_frame(&mut self) -> usize {
        let mut encoder = SliceWriter { buffer: &mut self.buf, pos: 0 };
        encoder.begin_frame(b'H');

        encoder.write_signed_vb(self.gps_state.home_latitude_degrees_1e7);
        encoder.write_signed_vb(self.gps_state.home_longitude_degrees_1e7);

        encoder.write_signed_vb(self.gps_state.home_altitude_cm / 10);

        encoder.end_frame()
    }

    /// GPS frame: g_frame.
    pub fn log_g_frame(&mut self, current_time_us: u32) -> usize {
        let mut encoder = SliceWriter { buffer: &mut self.buf, pos: 0 };
        encoder.begin_frame(b'G');

        // If we're logging every frame, then a GPS frame always appears just after a frame with the
        // current_time timestamp in the log, so the reader can just use that timestamp for the GPS frame.
        // If we're not logging every frame, we need to store the time of this GPS frame.
        if self.condition_cache.test(FieldCondition::NOT_LOGGING_EVERY_FRAME) {
            // Predict the time of the last frame in the main log
            encoder.write_unsigned_vb(current_time_us - self.main_states[self.state_index_current].time_us);
        }

        encoder.write_unsigned_vb(u32::from(self.gps_state.satellite_count));
        encoder.write_signed_vb(self.gps_state.latitude_degrees_1e7 - self.home_latitude_degrees_1e7);
        encoder.write_signed_vb(self.gps_state.longitude_degrees_1e7 - self.home_longitude_degrees_1e7);
        // log altitude in increments of 0.1m
        encoder.write_signed_vb(self.gps_state.altitude_cm / 10);

        #[allow(clippy::cast_sign_loss)]
        if self.config.gps_use_3d_speed {
            encoder.write_unsigned_vb(self.gps_state.speed3d_cmps as u32);
        } else {
            encoder.write_unsigned_vb(self.gps_state.ground_speed_cmps as u32);
        }

        #[allow(clippy::cast_sign_loss)]
        encoder.write_unsigned_vb(self.gps_state.ground_course_deci_degrees as u32);

        encoder.write_signed_vb_16(self.gps_state.velocity_north_cmps);
        encoder.write_signed_vb_16(self.gps_state.velocity_east_cmps);
        encoder.write_signed_vb_16(self.gps_state.velocity_down_cmps);

        encoder.end_frame()
    }
}

impl Blackbox {
    #[allow(clippy::too_many_lines)]
    /// Intra frame: i_frame.
    pub fn log_i_frame(&mut self) -> usize {
        let mut encoder = SliceWriter { buffer: &mut self.buf, pos: 0 };
        encoder.begin_frame(b'I');

        assert_i_field_encoding!("loopIteration", FieldPredictor::ZERO, FieldEncoding::UNSIGNED_VB);
        encoder.write_unsigned_vb(self.iteration);

        let current = &self.main_states[self.state_index_current];

        encoder.write_unsigned_vb(current.time_us);

        if self.condition_cache.test(FieldCondition::PID) {
            assert_i_field_encoding!("axisP", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            encoder.write_signed_vb_array(&current.pid_p);
            assert_i_field_encoding!("axisI", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            encoder.write_signed_vb_array(&current.pid_i);

            assert_i_field_encoding!("axisD", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            if self.condition_cache.test(FieldCondition::PID_D_ROLL) {
                encoder.write_signed_vb(current.pid_d[0]);
            }
            if self.condition_cache.test(FieldCondition::PID_D_PITCH) {
                encoder.write_signed_vb(current.pid_d[1]);
            }
            if self.condition_cache.test(FieldCondition::PID_D_YAW) {
                encoder.write_signed_vb(current.pid_d[2]);
            }

            assert_i_field_encoding!("axisF", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            if self.condition_cache.test(FieldCondition::PID_K) {
                encoder.write_signed_vb_array(&current.pid_k);
            }

            assert_i_field_encoding!("axisS", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            if self.condition_cache.test(FieldCondition::PID_S_ROLL) {
                encoder.write_signed_vb(current.pid_s[0]);
            }
            if self.condition_cache.test(FieldCondition::PID_S_PITCH) {
                encoder.write_signed_vb(current.pid_s[1]);
            }
            if self.condition_cache.test(FieldCondition::PID_S_YAW) {
                encoder.write_signed_vb(current.pid_s[2]);
            }

            assert_i_field_encoding!("rc_command", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            if self.condition_cache.test(FieldCondition::RC_COMMANDS) {
                // Write roll, pitch and yaw first, these are signed values in the range [-500,500]
                let rc_commands = [current.rc_commands[0], current.rc_commands[1], current.rc_commands[2]];
                encoder.write_signed_vb_16_array(&rc_commands);

                // Write the throttle separately from the rest of the RC data as it's UNSIGNED.
                // Throttle lies in range [PWM_RANGE_MIN, PWM_RANGE_MAX], ie [1000, 2000]
                #[allow(clippy::cast_sign_loss)]
                encoder.write_unsigned_vb((current.rc_commands[MainState::THROTTLE] - self.min_throttle) as u32);
            }

            assert_i_field_encoding!("setpoint", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            if self.condition_cache.test(FieldCondition::SETPOINT) {
                // Write setpoint roll, pitch, yaw, and throttle
                encoder.write_signed_vb_16_array(&current.setpoints);
            }

            assert_i_field_encoding!("vbat_latest", FieldPredictor::VBATREF, FieldEncoding::NEG_14BIT);
            if self.condition_cache.test(FieldCondition::BATTERY_VOLTAGE) {
                //Our voltage is expected to decrease over the course of the flight, so store our difference from
                //the reference:
                // Write 14 bits even if the number is negative (which would otherwise result in 32 bits)
                encoder.write_unsigned_vb(u32::from(self.vbat_reference - current.battery_voltage) & 0x3FFF);
            }

            assert_i_field_encoding!("amperage_latest", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            if self.condition_cache.test(FieldCondition::BATTERY_CURRENT) {
                // 12bit value directly from ADC
                encoder.write_unsigned_vb_16(current.amperage);
            }

            #[cfg(feature = "magnetometer")]
            if self.condition_cache.test(FieldCondition::MAGNETOMETER) {
                encoder.write_signed_vb_16_array(&current.mag);
            }

            #[cfg(feature = "barometer")]
            if self.condition_cache.test(FieldCondition::BAROMETER) {
                encoder.write_signed_vb(current.baro_altitude);
            }

            #[cfg(feature = "rangefinder")]
            if self.condition_cache.test(FieldCondition::RANGEFINDER) {
                encoder.write_signed_vb(current.range_raw);
            }

            if self.condition_cache.test(FieldCondition::RSSI) {
                encoder.write_unsigned_vb_16(current.rssi);
            }

            assert_i_field_encoding!("gyro_adc", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            if self.condition_cache.test(FieldCondition::GYRO) {
                encoder.write_signed_vb_16_array(&current.gyro);
            }

            assert_i_field_encoding!("gyroUnfilt", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            if self.condition_cache.test(FieldCondition::GYRO_UNFILTERED) {
                encoder.write_signed_vb_16_array(&current.gyro_unfiltered);
            }

            assert_i_field_encoding!("accSmooth", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            if self.condition_cache.test(FieldCondition::ACC) {
                encoder.write_signed_vb_16_array(&current.acc);
            }

            assert_i_field_encoding!("imuQuaternion", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            if self.condition_cache.test(FieldCondition::ATTITUDE) {
                encoder.write_signed_vb_16_array(&current.orientation);
            }

            assert_i_field_encoding!("debug", FieldPredictor::ZERO, FieldEncoding::SIGNED_VB);
            if self.condition_cache.test(FieldCondition::DEBUG) {
                encoder.write_signed_vb_16_array(&current.debug);
            }

            assert_i_field_encoding!("motor", FieldPredictor::MIN_MOTOR, FieldEncoding::UNSIGNED_VB);
            if Self::field_enabled(self.log_select_enabled, LogFieldSelect::MOTOR) {
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
            if Self::field_enabled(self.log_select_enabled, LogFieldSelect::MOTOR) {
                for erpm in current.erpm {
                    encoder.write_signed_vb_16(erpm);
                }
            }
        }

        let ret = encoder.end_frame();
        // Rotate the state indices
        let new_current = self.state_index_pre_previous;
        self.state_index_pre_previous = self.state_index_previous;
        self.state_index_previous = self.state_index_current;
        self.state_index_current = new_current;
        // This is an i_frame, so there is no other pre_previous state, so we copy the previous state into the pre_previous state
        self.main_states[self.state_index_pre_previous] = self.main_states[self.state_index_previous];

        ret
    }
}

impl Blackbox {
    /// Write a Predictor frame (p_frame).
    /// Note: the predictions are hard coded to match the values defined in BLACKBOX_MAIN_FIELDS:
    /// the code is made safe by asserting the p_encoding values.
    /// So this code and those definitions must be changed in tandem with each other.
    #[allow(clippy::too_many_lines)]
    pub fn log_p_frame(&mut self) -> usize {
        let mut encoder = SliceWriter { buffer: &mut self.buf, pos: 0 };
        encoder.begin_frame(b'P');

        let current = &self.main_states[self.state_index_current];
        let previous = &self.main_states[self.state_index_previous];
        let pre_previous = &self.main_states[self.state_index_pre_previous];

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
        if self.condition_cache.test(FieldCondition::PID) {
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
            if self.condition_cache.test(FieldCondition::PID_D_ROLL) {
                encoder.write_signed_vb(current.pid_d[0] - previous.pid_d[0]);
            }
            if self.condition_cache.test(FieldCondition::PID_D_PITCH) {
                encoder.write_signed_vb(current.pid_d[1] - previous.pid_d[1]);
            }
            if self.condition_cache.test(FieldCondition::PID_D_YAW) {
                encoder.write_signed_vb(current.pid_d[2] - previous.pid_d[2]);
            }

            if self.condition_cache.test(FieldCondition::PID_K) {
                let deltas = [
                    current.pid_k[0] - previous.pid_k[0],
                    current.pid_k[1] - previous.pid_k[1],
                    current.pid_k[2] - previous.pid_k[2],
                ];
                encoder.write_signed_vb_array(&deltas);
            }

            if self.condition_cache.test(FieldCondition::PID_S_ROLL) {
                encoder.write_signed_vb(current.pid_s[0] - previous.pid_s[0]);
            }
            if self.condition_cache.test(FieldCondition::PID_S_PITCH) {
                encoder.write_signed_vb(current.pid_s[1] - previous.pid_s[1]);
            }
            if self.condition_cache.test(FieldCondition::PID_S_YAW) {
                encoder.write_signed_vb(current.pid_s[2] - previous.pid_s[2]);
            }
        }

        // RC tends to stay the same or fairly small for many frames at a time, so use an encoding that
        assert_p_field_encoding!("rc_command", FieldPredictor::PREVIOUS, FieldEncoding::TAG8_4S16);
        if self.condition_cache.test(FieldCondition::RC_COMMANDS) {
            let deltas = [
                current.rc_commands[0] - previous.rc_commands[0],
                current.rc_commands[1] - previous.rc_commands[1],
                current.rc_commands[2] - previous.rc_commands[2],
                current.rc_commands[3] - previous.rc_commands[3],
            ];
            encoder.write_tag8_4s16(deltas);
        }
        assert_p_field_encoding!("setpoint", FieldPredictor::PREVIOUS, FieldEncoding::TAG8_4S16);
        if self.condition_cache.test(FieldCondition::SETPOINT) {
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

        if self.condition_cache.test(FieldCondition::BATTERY_VOLTAGE) {
            deltas[optional_field_count] = i32::from(current.battery_voltage - previous.battery_voltage);
            optional_field_count += 1;
        }

        if self.condition_cache.test(FieldCondition::BATTERY_CURRENT) {
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
        if self.condition_cache.test(FieldCondition::BAROMETER) {
            deltas[optional_field_count] = current.baro_altitude - previous.baro_altitude;
            optional_field_count += 1;
        }

        #[cfg(feature = "rangefinder")]
        if self.condition_cache.test(FieldCondition::RANGEFINDER) {
            deltas[optional_field_count] = current.range_raw - previous.range_raw;
            optional_field_count += 1;
        }

        if self.condition_cache.test(FieldCondition::RSSI) {
            deltas[optional_field_count] = i32::from(current.rssi - previous.rssi);
        }

        assert_p_field_encoding!("vbat_latest", FieldPredictor::PREVIOUS, FieldEncoding::TAG8_8SVB);
        assert_p_field_encoding!("amperage_latest", FieldPredictor::PREVIOUS, FieldEncoding::TAG8_8SVB);
        encoder.write_tag8_8svb(&deltas);

        // Since gyros, accelerometers and motors are noisy, base their predictions on the average of the history:
        assert_p_field_encoding!("gyro_adc", FieldPredictor::AVERAGE_2, FieldEncoding::SIGNED_VB);
        if self.condition_cache.test(FieldCondition::GYRO) {
            for ii in 0..MainState::XYZ_AXIS_COUNT {
                let predicted = i16::midpoint(previous.gyro[ii], pre_previous.gyro[ii]);
                encoder.write_signed_vb_16(current.gyro[ii] - predicted);
            }
        }
        assert_p_field_encoding!("gyroUnfilt", FieldPredictor::AVERAGE_2, FieldEncoding::SIGNED_VB);
        if self.condition_cache.test(FieldCondition::GYRO_UNFILTERED) {
            for ii in 0..MainState::XYZ_AXIS_COUNT {
                let predicted = i16::midpoint(previous.gyro_unfiltered[ii], pre_previous.gyro_unfiltered[ii]);
                encoder.write_signed_vb_16(current.gyro_unfiltered[ii] - predicted);
            }
        }
        assert_p_field_encoding!("accSmooth", FieldPredictor::AVERAGE_2, FieldEncoding::SIGNED_VB);
        if self.condition_cache.test(FieldCondition::ACC) {
            for ii in 0..MainState::XYZ_AXIS_COUNT {
                let predicted = i16::midpoint(previous.acc[ii], pre_previous.acc[ii]);
                encoder.write_signed_vb_16(current.acc[ii] - predicted);
            }
        }
        assert_p_field_encoding!("imuQuaternion", FieldPredictor::AVERAGE_2, FieldEncoding::SIGNED_VB);
        if self.condition_cache.test(FieldCondition::ATTITUDE) {
            for ii in 0..MainState::XYZ_AXIS_COUNT {
                let predicted = i16::midpoint(previous.orientation[ii], pre_previous.orientation[ii]);
                encoder.write_signed_vb_16(current.orientation[ii] - predicted);
            }
        }

        assert_p_field_encoding!("debug", FieldPredictor::AVERAGE_2, FieldEncoding::SIGNED_VB);
        if self.condition_cache.test(FieldCondition::DEBUG) {
            for ii in 0..MainState::DEBUG_VALUE_COUNT {
                let predicted = i16::midpoint(previous.debug[ii], pre_previous.debug[ii]);
                encoder.write_signed_vb_16(current.debug[ii] - predicted);
            }
        }

        assert_p_field_encoding!("motor", FieldPredictor::AVERAGE_2, FieldEncoding::SIGNED_VB);
        if Self::field_enabled(self.log_select_enabled, LogFieldSelect::MOTOR) {
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
        if Self::field_enabled(self.log_select_enabled, LogFieldSelect::MOTOR_RPM) {
            for ii in 0..self.motor_count {
                encoder.write_signed_vb_16(current.erpm[ii] - previous.erpm[ii]);
            }
        }
        let ret = encoder.end_frame();

        self.logged_any_frames = true;
        // Rotate the state indices
        let new_current = self.state_index_pre_previous;
        self.state_index_pre_previous = self.state_index_previous;
        self.state_index_previous = self.state_index_current;
        self.state_index_current = new_current;

        ret
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    #![allow(unused_results)]

    #[allow(unused)]
    use super::*;

    fn is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    fn is_config<
        T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq + Serialize + for<'a> Deserialize<'a>,
    >() {
    }

    #[test]
    fn normal_types() {
        is_normal::<Blackbox>();
        is_normal::<BlackboxCallbacks>();
        is_full::<BlackboxStart>();
        is_config::<BlackboxConfig>();
    }
    #[test]
    fn new() {
        let blackbox = Blackbox::default();
        assert_eq!(0, blackbox.iteration);
    }
    #[test]
    fn i_encodings() {
        assert_i_field_encoding!("loopIteration", FieldPredictor::ZERO, FieldEncoding::UNSIGNED_VB);
        let mut blackbox = Blackbox::default();
        assert_eq!(0, blackbox.state_index_current);
        assert_eq!(1, blackbox.state_index_previous);
        assert_eq!(2, blackbox.state_index_pre_previous);
        blackbox.log_i_frame();
        assert_eq!(2, blackbox.state_index_current);
        assert_eq!(0, blackbox.state_index_previous);
        assert_eq!(1, blackbox.state_index_pre_previous);
    }
    #[test]
    fn p_encodings() {
        assert_p_field_encoding!("loopIteration", FieldPredictor::INC, FieldEncoding::ZERO);
        let mut blackbox = Blackbox::default();
        assert_eq!(0, blackbox.state_index_current);
        assert_eq!(1, blackbox.state_index_previous);
        assert_eq!(2, blackbox.state_index_pre_previous);
        blackbox.log_p_frame();
        assert_eq!(2, blackbox.state_index_current);
        assert_eq!(0, blackbox.state_index_previous);
        assert_eq!(1, blackbox.state_index_pre_previous);
    }
}
