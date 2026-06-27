use crate::{
    BlackboxEvent, SliceEncoder,
    data::{BlackboxMainData, BlackboxSlowData},
    field_definitions::{FieldCondition, FieldSelect},
    logger_state::LoggerState,
};

#[cfg(feature = "gps")]
use crate::data::{BlackboxGpsData, BlackboxGpsPosition};

use simple_bitset::BitSet64;

/// System info.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SysInfo {
    pub(crate) looptime: u32,
    pub motor_output_min: u16,
    pub motor_output_max: u16,
}

impl SysInfo {
    pub const fn new() -> Self {
        Self {
            looptime: 125, // 125us = 8kHz gyro/pid loop
            motor_output_min: 158,
            motor_output_max: 2047,
        }
    }
}
impl Default for SysInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Blackbox logger.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Logger {
    pub(crate) conditions: BitSet64,
    pub(crate) enabled_fields: u32,

    pub(crate) last_arming_beep_time_us: u32,
    pub(crate) last_flight_mode_flags: u32,
    pub(crate) i_interval: u32,
    pub(crate) p_interval: u32,
    s_interval: u32,
    i_frame_index: u32,
    p_frame_index: u32,
    pub(crate) s_frame_index: u32,
    pub(crate) iteration: u32,

    pub(crate) logged_any_frames: bool,
    pub(crate) new_gps_data: bool,

    pub(crate) motor_count: usize,
    pub(crate) servo_count: usize,
    pub(crate) debug_mode: u16,
    pub(crate) min_throttle: u16,
    pub(crate) max_throttle: u16,
    pub(crate) vbat_reference: u16,

    pub(crate) slow_data: BlackboxSlowData,
    #[cfg(feature = "gps")]
    pub(crate) gps_data: BlackboxGpsData,
    #[cfg(feature = "gps")]
    pub(crate) gps_home: BlackboxGpsPosition,
    pub sys_info: SysInfo,

    pub(crate) main_data: [BlackboxMainData; 3],
    pub(crate) main_data_current_idx: usize,
    pub(crate) main_data_previous_idx: usize,
    pub(crate) main_data_pre_previous_idx: usize,
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

impl Logger {
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            motor_count: 4,
            servo_count: 0,
            debug_mode: 0,
            min_throttle: 1070,
            max_throttle: 2000,
            vbat_reference: 2466,
            conditions: BitSet64::new(),

            p_interval: 8,   // 8*125us = 1000us = 1kHz logging
            i_interval: 256, // 256*p_interval = 256ms
            s_interval: 0,   // set to 256*i_interval in init. 256*256ms = 65.536s, or approximately one a minute
            i_frame_index: 0,
            p_frame_index: 0,
            s_frame_index: 0,
            iteration: 0,

            enabled_fields: 0,
            last_arming_beep_time_us: 0,
            last_flight_mode_flags: 0,
            logged_any_frames: false,
            new_gps_data: false,

            slow_data: BlackboxSlowData::new(),

            #[cfg(feature = "gps")]
            gps_data: BlackboxGpsData::new(),
            #[cfg(feature = "gps")]
            gps_home: BlackboxGpsPosition::new(),
            sys_info: SysInfo::new(),

            main_data: [BlackboxMainData::new(); 3],
            main_data_current_idx: 0,
            main_data_previous_idx: 1,
            main_data_pre_previous_idx: 2,
        }
    }
}

impl Logger {
    pub fn init(&mut self, sample_rate: u8, fields_disabled_mask: u32) {
        self.enabled_fields = FieldSelect::DEBUG
            | FieldSelect::PID
            | FieldSelect::PID_DTERM_ROLL
            | FieldSelect::PID_DTERM_PITCH
            | FieldSelect::PID_STERM_ROLL
            | FieldSelect::PID_STERM_PITCH
            | FieldSelect::PID_STERM_YAW
            | FieldSelect::PID_KTERM
            | FieldSelect::SETPOINT
            | FieldSelect::RC_COMMANDS
            | FieldSelect::RSSI
            | FieldSelect::GYRO
            | FieldSelect::GYRO_UNFILTERED
            | FieldSelect::ATTITUDE
            | FieldSelect::MOTOR
            | FieldSelect::BATTERY_VOLTAGE
            | FieldSelect::BATTERY_CURRENT
            | FieldSelect::BAROMETER
            | FieldSelect::RANGEFINDER
            | FieldSelect::MAGNETOMETER
            | FieldSelect::ACCELEROMETER;

        #[cfg(feature = "dshot_telemetry")]
        {
            self.enabled_fields |= FieldSelect::MOTOR_RPM;
        }
        #[cfg(feature = "gps")]
        {
            self.enabled_fields |= FieldSelect::GPS;
        }
        self.enabled_fields &= !fields_disabled_mask;

        self.build_field_condition_cache();
        //self.conditions &= !BitSet64::from(config.fields_disabled_mask);

        // an i_frame is written every 32ms
        // update() is run in synchronization with the PID loop
        // target_pid_looptime_us is 1000 for 1kHz loop, 500 for 2kHz loop etc, target_pid_looptime_us is rounded for short looptimes
        // TODO: self.i_interval = 32 * 1000 / self.target_pid_looptime_us;

        self.p_interval = 1 << sample_rate;
        if self.p_interval > self.i_interval {
            self.p_interval = 0; // log only i_frames if logging frequency is too low
        }

        // s_frame is written every 256 i_frames, ie every 256*32 = 8192ms, approx every 8 seconds
        self.s_interval = self.i_interval * 256;

        /*if config.device == BlackboxDevice::NONE {
            self.set_data(STATE_DISABLED);
        } else if (config.mode == BlackboxMode::ALWAYS_ON) {
            self.start();
        } else {
            self.set_data(STATE_STOPPED);
        }*/
        self.reset_iteration_timers(); // should be done after s_interval is set
    }
}

impl Logger {
    /// Returns true if `a` is after `b`, taking account of wrapping at `u32::MAX`.
    /// ```
    /// # use blackbox_logger::Logger;
    /// assert!(Logger::is_after(10, 9));
    /// assert!(Logger::is_after(0, u32::MAX));
    /// assert!(Logger::is_after(1, u32::MAX));
    /// ```
    #[must_use]
    pub fn is_after(a: u32, b: u32) -> bool {
        // This calculates (a - b) mod 2^32
        a.wrapping_sub(b) < (u32::MAX / 2)
    }
    // Some of the main data comes from the GyroPidMessage, some comes from teh SetpointMessage

    #[inline]
    pub fn set_main_data(&mut self, _current_time_us: u32, main_data: BlackboxMainData) {
        self.main_data[self.main_data_current_idx] = main_data;
    }
    #[inline]
    pub fn set_slow_data(&mut self, slow_data: BlackboxSlowData) {
        // TODO: need to check time
        self.slow_data = slow_data;
    }

    #[cfg(feature = "gps")]
    #[inline]
    pub fn set_gps_data(&mut self, gps_data: BlackboxGpsData) {
        self.new_gps_data = self.gps_data != gps_data;
        self.gps_data = gps_data;
    }

    pub fn update(
        &mut self,
        state: &mut LoggerState,
        encoder: &mut SliceEncoder,
        current_time_us: u32,
        is_active: bool,
    ) -> usize {
        state.update(self, encoder, current_time_us, is_active)
    }

    /// Called when the flight controller signals it has new data.
    pub fn log_iteration(&mut self, encoder: &mut SliceEncoder, current_time_us: u32) {
        self.logged_any_frames = true;

        self.main_data[self.main_data_current_idx].time_us = current_time_us;

        // Write a keyframe every i_interval frames so we can resynchronise upon missing frames
        if self.should_log_i_frame() {
            self.log_i_frame(encoder);
            if self.is_only_logging_i_frames() {
                // other frames are normally logged alongside p_frames, however if we are only logging i_frames then we need to log them here.
                if self.should_log_s_frame() {
                    self.log_s_frame(encoder);
                }
                self.log_event_arming_beep_if_needed(encoder, 0);
                //self.log_event_flight_mode_if_needed(encoder); // Check for FlightMode status change event
            }
        } else {
            self.log_event_arming_beep_if_needed(encoder, 0);
            //self.log_event_flight_mode_if_needed(encoder); // Check for FlightMode status change event

            if self.should_log_p_frame() {
                // Log s_frame alongside p_frame.
                if self.should_log_s_frame() {
                    self.log_s_frame(encoder);
                }
                self.log_p_frame(encoder);
            }
            #[cfg(feature = "gps")]
            if self.should_log_h_frame() {
                self.log_h_frame(encoder);
                self.log_g_frame(encoder, current_time_us);
            } else if self.should_log_g_frame() {
                self.log_g_frame(encoder, current_time_us);
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn should_log_i_frame(&self) -> bool {
        self.i_frame_index == 0
    }
    #[inline]
    #[must_use]
    pub fn should_log_h_frame(&self) -> bool {
        (self.enabled_fields & FieldSelect::GPS != 0) && self.new_gps_data
    }
    #[inline]
    #[must_use]
    pub fn should_log_g_frame(&self) -> bool {
        (self.enabled_fields & FieldSelect::GPS != 0) && self.new_gps_data
    }
    #[inline]
    #[must_use]
    pub fn should_log_p_frame(&self) -> bool {
        self.p_frame_index == 0 && !self.is_only_logging_i_frames()
    }
    /// If the data in the slow frame has changed, log a slow frame.
    ///
    /// The frame is also logged if it has been more than `s_interval` logging iterations
    /// since the field was last logged.
    // Write the slow frame periodically so it can be recovered if we ever lose sync
    #[inline]
    #[must_use]
    pub fn should_log_s_frame(&self) -> bool {
        self.s_frame_index >= self.s_interval
    }
    #[inline]
    #[must_use]
    pub fn is_only_logging_i_frames(&self) -> bool {
        self.p_interval == 0
    }

    /// If an arming beep has played since it was last logged, write the time of the arming beep to the log as a synchronization point.
    pub fn log_event_arming_beep_if_needed(&mut self, encoder: &mut SliceEncoder, arming_beep_time_us: u32) {
        if arming_beep_time_us != self.last_arming_beep_time_us {
            self.last_arming_beep_time_us = arming_beep_time_us;
            let event = BlackboxEvent::SyncBeep(arming_beep_time_us);
            self.log_e_frame(encoder, event);
        }
    }
    pub fn log_event_flight_mode_if_needed(&mut self, encoder: &mut SliceEncoder, rc_mode_activation_mask: u32) {
        if rc_mode_activation_mask != self.last_flight_mode_flags {
            let event = BlackboxEvent::FlightMode(rc_mode_activation_mask, self.last_flight_mode_flags);
            self.log_e_frame(encoder, event);
            self.last_flight_mode_flags = rc_mode_activation_mask;
        }
    }
}

impl Logger {
    pub fn reset_iteration_timers(&mut self) {
        self.iteration = 0;
        self.i_frame_index = 0;
        self.p_frame_index = 0;
        self.s_frame_index = self.s_interval; // so s_frame written next iteration
    }

    /// Called once every FC loop in order to keep track of how many FC loop iterations have passed.
    pub fn advance_iteration_timers(&mut self) {
        self.iteration = self.iteration.wrapping_add(1);
        self.s_frame_index = self.s_frame_index.wrapping_add(1);

        self.i_frame_index = self.i_frame_index.wrapping_add(1);
        if self.i_frame_index >= self.i_interval {
            self.i_frame_index = 0; // value of zero means i_frame will be written on next update
            self.p_frame_index = 0;
        } else {
            self.p_frame_index = self.p_frame_index.wrapping_add(1);
            if self.p_frame_index >= self.p_interval {
                self.p_frame_index = 0; // value of zero means p_frame will be written on next update, if i_frame not written
            }
        }
    }

    /// Build condition cache, called from `start()`.
    pub fn build_field_condition_cache(&mut self) {
        self.conditions.reset_all();
        for condition in FieldCondition::FIRST..FieldCondition::LAST {
            if self.test_field_condition_uncached(condition) {
                self.conditions.set(condition);
            }
        }
    }

    /// Helper function to check if a field is enabled.
    #[inline]
    #[must_use]
    pub fn field_enabled(enabled_mask: u32, field: u32) -> bool {
        enabled_mask & field != 0
    }

    /// Public method to check if a log field is enabled.
    #[inline]
    #[must_use]
    pub fn is_field_enabled(&self, field: u32) -> bool {
        Self::field_enabled(self.enabled_fields, field)
    }

    // Called from build_field_condition_cache(), which is called from start()
    // Test condition without caching
    #[must_use]
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
                self.is_field_enabled(FieldSelect::MOTOR)
                    && self.motor_count > (condition - FieldCondition::AT_LEAST_MOTORS_1) as usize
            }

            #[cfg(feature = "dshot_telemetry")]
            FieldCondition::MOTOR_1_HAS_RPM
            | FieldCondition::MOTOR_2_HAS_RPM
            | FieldCondition::MOTOR_3_HAS_RPM
            | FieldCondition::MOTOR_4_HAS_RPM
            | FieldCondition::MOTOR_5_HAS_RPM
            | FieldCondition::MOTOR_6_HAS_RPM
            | FieldCondition::MOTOR_7_HAS_RPM
            | FieldCondition::MOTOR_8_HAS_RPM => {
                self.is_field_enabled(FieldSelect::MOTOR_RPM)
                    && self.motor_count > (condition - FieldCondition::MOTOR_1_HAS_RPM) as usize
            }

            FieldCondition::SERVOS => self.is_field_enabled(FieldSelect::SERVO) && self.servo_count > 0,

            FieldCondition::PID => self.is_field_enabled(FieldSelect::PID),

            FieldCondition::PID_K => {
                self.is_field_enabled(FieldSelect::PID) && self.is_field_enabled(FieldSelect::PID_KTERM)
            }
            FieldCondition::PID_D_ROLL => {
                self.is_field_enabled(FieldSelect::PID) && self.is_field_enabled(FieldSelect::PID_DTERM_ROLL)
            }
            FieldCondition::PID_D_PITCH => {
                self.is_field_enabled(FieldSelect::PID) && self.is_field_enabled(FieldSelect::PID_DTERM_PITCH)
            }
            FieldCondition::PID_D_YAW => {
                self.is_field_enabled(FieldSelect::PID) && self.is_field_enabled(FieldSelect::PID_DTERM_YAW)
            }
            FieldCondition::PID_S_ROLL => {
                self.is_field_enabled(FieldSelect::PID) && self.is_field_enabled(FieldSelect::PID_STERM_ROLL)
            }
            FieldCondition::PID_S_PITCH => {
                self.is_field_enabled(FieldSelect::PID) && self.is_field_enabled(FieldSelect::PID_STERM_PITCH)
            }
            FieldCondition::PID_S_YAW => {
                self.is_field_enabled(FieldSelect::PID) && self.is_field_enabled(FieldSelect::PID_STERM_YAW)
            }

            FieldCondition::RC_COMMANDS => self.is_field_enabled(FieldSelect::RC_COMMANDS),
            FieldCondition::SETPOINT => self.is_field_enabled(FieldSelect::SETPOINT),
            FieldCondition::MAGNETOMETER => self.is_field_enabled(FieldSelect::MAGNETOMETER),
            FieldCondition::BAROMETER => self.is_field_enabled(FieldSelect::BAROMETER),
            FieldCondition::BATTERY_VOLTAGE => self.is_field_enabled(FieldSelect::BATTERY_VOLTAGE),
            FieldCondition::BATTERY_CURRENT => self.is_field_enabled(FieldSelect::BATTERY_CURRENT),
            FieldCondition::RANGEFINDER => self.is_field_enabled(FieldSelect::RANGEFINDER),
            FieldCondition::RSSI => self.is_field_enabled(FieldSelect::RSSI),

            FieldCondition::NOT_LOGGING_EVERY_FRAME => self.p_interval != self.i_interval,

            FieldCondition::GYRO => self.is_field_enabled(FieldSelect::GYRO),
            FieldCondition::GYRO_UNFILTERED => self.is_field_enabled(FieldSelect::GYRO_UNFILTERED),
            FieldCondition::ACC => self.is_field_enabled(FieldSelect::ACCELEROMETER),
            FieldCondition::ATTITUDE => self.is_field_enabled(FieldSelect::ATTITUDE),

            FieldCondition::DEBUG => self.is_field_enabled(FieldSelect::DEBUG) && self.debug_mode != 0,

            // Handle any unknown condition
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<Logger>();
        is_full::<SysInfo>();
    }
    #[test]
    fn new() {
        let logger = Logger::new();
        assert_eq!(4, logger.motor_count);
    }
}
