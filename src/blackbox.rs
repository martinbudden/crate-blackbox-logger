use crate::BlackboxTelemetry;
use crate::{FieldCondition, LogFieldSelect};
use crate::{GpsState, MainState, SlowState};
use serde::{Deserialize, Serialize};
use vqm::BitSet64;

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
    pub fn new() -> Self {
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
    pub fn new() -> Self {
        Self { debug_mode: 0, motor_count: 4, servo_count: 0 }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Blackbox {
    pub(crate) iteration: u32,
    loop_index: u32,

    pub(crate) motor_count: usize,
    pub(crate) servo_count: usize,
    pub(crate) debug_mode: u32,
    pub(crate) motor_output_min: i16,
    pub(crate) min_throttle: i16,
    pub(crate) vbat_reference: u16,
    pub(crate) logged_any_frames: bool,

    i_frame_index: u32,
    i_interval: u32,
    p_frame_index: u32,
    p_interval: u32,
    pub(crate) s_frame_index: u32,
    s_interval: u32,

    pub(crate) conditions: BitSet64,
    pub(crate) log_select_enabled: u32,

    pub(crate) slow_state: SlowState,
    pub(crate) gps_state: GpsState,
    pub(crate) home_longitude_degrees_1e7: i32, // home longitude in degrees * 1e7
    pub(crate) home_latitude_degrees_1e7: i32,  // home latitude in degrees * 1e7
    pub(crate) home_altitude_cm: i32,           // home altitude in cm

    pub(crate) main_states: [MainState; 3],
    pub(crate) state_index_current: usize,
    pub(crate) state_index_previous: usize,
    pub(crate) state_index_pre_previous: usize,
    pub(crate) config: BlackboxConfig,
    pub(crate) buf: [u8; 1024],
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
            conditions: BitSet64::default(),
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
        | LogFieldSelect::ATTITUDE
        | LogFieldSelect::MOTOR
        | LogFieldSelect::MOTOR_RPM
        | LogFieldSelect::BATTERY_VOLTAGE
        | LogFieldSelect::BATTERY_CURRENT;

        self.build_field_condition_cache();
        //self.conditions &= !BitSet64::from(config.fields_disabled_mask);

        self.reset_iteration_timers();

        // an i_frame is written every 32ms
        // blackboxUpdate() is run in synchronization with the PID loop
        // target_pid_looptime_us is 1000 for 1kHz loop, 500 for 2kHz loop etc, target_pid_looptime_us is rounded for short looptimes
        // TODO: self.i_interval = 32 * 1000 / self.target_pid_looptime_us;

        self.p_interval = 1 << config.sample_rate;
        if self.p_interval > self.i_interval {
            self.p_interval = 0; // log only i_frames if logging frequency is too low
        }

        // s_frame is written every 256*32 = 8192ms, approx every 8 seconds
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
        self.conditions.reset_all();
        for condition in FieldCondition::FIRST..FieldCondition::LAST {
            if self.test_field_condition_uncached(condition) {
                _ = self.conditions.set(condition);
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
    pub fn load_main_state(&mut self, current_time_us: u32, telemetry: BlackboxTelemetry) {
        let current = &mut self.main_states[self.state_index_current];
        current.time_us = current_time_us;
        current.acc = (telemetry.acc * 4096.0).into();
        current.gyro = (telemetry.gyro_rps.to_degrees()).into();
        current.gyro_unfiltered = (telemetry.gyro_rps_unfiltered.to_degrees()).into();
    }

    // Callbacks are probably better done with Generics, but this is simpler for now.
    // TODO: Actually, on second thought, it is probably better if this just peeks at the relevant watchers.
    /// Called when the flight controller signals it has new data.
    pub fn log_iteration(&mut self, current_time_us: u32, telemetry: BlackboxTelemetry) {
        // Write a keyframe every i_interval frames so we can resynchronise upon missing frames
        if self.should_log_i_frame() {
            // ie _loop_index == 0
            // Don't log a slow frame if the slow data didn't change (i_frames are already large enough without adding
            // an additional item to write at the same time). Unless we're *only* logging i_frames, then we have no choice.
            if self.is_only_logging_i_frames() {
                let _len = self.log_s_frame_if_needed();
                //self.sd_card.write_all(&self.buf[..len]).await.ok();
            }

            self.load_main_state(current_time_us, telemetry);
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

                self.load_main_state(current_time_us, telemetry);
                let _len = self.log_p_frame();
                //self.sd_card.write_all(&self.buf[..len]).await.ok();
            }
            #[cfg(feature = "gps")]
            if Self::field_enabled(self.log_select_enabled, LogFieldSelect::GPS) {
                let gps_state_new = GpsState::new();

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
        // TODO: add a check if new slow data has arrived
        if self.s_frame_index >= self.s_interval {
            return self.log_s_frame();
        }
        0
    }
}

impl Blackbox {
    //fn field_enabled(enabled_mask:u32, field:LogFieldSelect) -> bool { enabled_mask & (field as u32) }
    //pub fn is_field_enabled(&self, field:LogFieldSelect) ->bool { field_enabled(self.log_select_enabled, field) }
    // Helper function to check if a field is enabled
    pub(crate) fn field_enabled(enabled_mask: u32, field: u32) -> bool {
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
        is_full::<BlackboxStart>();
        is_config::<BlackboxConfig>();
    }
    #[test]
    fn new() {
        let blackbox = Blackbox::default();
        assert_eq!(0, blackbox.iteration);
    }
}
