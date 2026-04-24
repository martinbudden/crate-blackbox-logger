use crate::field_arrays::BLACKBOX_SLOW_FIELDS;
use crate::field_definitions::{FieldCondition, LogFieldSelect, MainFieldDefinition, SimpleFieldDefinition};
use crate::encoding::write_field_line;
use crate::states::{GpsState, MainState, SlowState};
use crate::{BlackboxSlowTelemetry, BlackboxTelemetry};
use crate::{BlackboxStartParameters, BlackboxWriter, Features, SliceWriter};
use vqm::BitSet64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blackbox {
    pub(crate) motor_count: usize,
    pub(crate) servo_count: usize,
    pub(crate) debug_mode: u32,
    pub(crate) motor_output_min: i16,
    pub(crate) min_throttle: i16,
    pub(crate) vbat_reference: u16,
    pub(crate) logged_any_frames: bool,
    pub(crate) conditions: BitSet64,
    features: Features,
    looptime: u32,
    loop_index: u32,
    i_interval: u32,
    p_interval: u32,
    s_interval: u32,
    i_frame_index: u32,
    p_frame_index: u32,
    pub(crate) s_frame_index: u32,
    pub(crate) iteration: u32,

    pub(crate) log_select_enabled: u32,
    pub(crate) new_slow_state: bool,
    pub(crate) new_gps_state: bool,
    pub(crate) slow_state: SlowState,
    pub(crate) gps_state: GpsState,
    pub(crate) home_longitude_degrees_1e7: i32, // home longitude in degrees * 1e7
    pub(crate) home_latitude_degrees_1e7: i32,  // home latitude in degrees * 1e7
    pub(crate) home_altitude_cm: i32,           // home altitude in cm

    pub(crate) main_states: [MainState; 3],
    pub(crate) main_state_index_current: usize,
    pub(crate) main_state_index_previous: usize,
    pub(crate) main_state_index_pre_previous: usize,
}

impl Default for Blackbox {
    fn default() -> Self {
        Self::new()
    }
}

impl Blackbox {
    pub fn new() -> Self {
        Self {
            motor_count: 4,
            servo_count: 0,
            debug_mode: 0,
            motor_output_min: 750,
            min_throttle: 700,
            vbat_reference: 0,
            logged_any_frames: false,
            conditions: BitSet64::default(),
            i_interval: 256,
            p_interval: 8,
            looptime: 125,
            log_select_enabled: 0,
            i_frame_index: 0,
            p_frame_index: 0,
            s_frame_index: 0,
            s_interval: 0,
            new_slow_state: false,
            new_gps_state: false,
            iteration: 0,
            loop_index: 0,
            features: Features {flags:Features::VBAT |Features::INFLIGHT_ACC_CAL |Features::RX_SERIAL |Features::BLACKBOX |Features::FAILSAFE},
            slow_state: SlowState::default(),
            gps_state: GpsState::default(),
            home_longitude_degrees_1e7: 0,
            home_latitude_degrees_1e7: 0,
            home_altitude_cm: 0,
            main_states: <[MainState; 3]>::default(),
            main_state_index_current: 0,
            main_state_index_previous: 1,
            main_state_index_pre_previous: 2,
        }
    }
}

impl Blackbox {
    /// Build condition cache, called from start().
    pub fn build_field_condition_cache(&mut self) {
        self.conditions.reset_all();
        for condition in FieldCondition::FIRST..FieldCondition::LAST {
            if self.test_field_condition_uncached(condition) {
                _ = self.conditions.set(condition);
            }
        }
    }
}

impl Blackbox {
    pub fn init(&mut self, sample_rate: u8) {
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

        self.p_interval = 1 << sample_rate;
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
}

impl Blackbox {
    pub fn load_telemetry(&mut self, current_time_us: u32, telemetry: BlackboxTelemetry) {
        let current = &mut self.main_states[self.main_state_index_current];
        current.time_us = current_time_us;
        current.acc = (telemetry.acc * 4096.0).into();
        current.gyro = (telemetry.gyro_rps.to_degrees()).into();
        current.gyro_unfiltered = (telemetry.gyro_rps_unfiltered.to_degrees()).into();
    }

    pub fn load_slow_telemetry(&mut self, telemetry: BlackboxSlowTelemetry) {
        self.new_slow_state = true;
        self.slow_state.flight_mode_flags = telemetry.flight_mode_flags;
        self.slow_state.state_flags = telemetry.state_flags;
        self.slow_state.failsafe_phase = telemetry.failsafe_phase;
        self.slow_state.rx_signal_received = telemetry.rx_signal_received;
        self.slow_state.rx_flight_channel_is_valid = telemetry.rx_flight_channel_is_valid;
    }

    pub fn load_gps_state(&mut self) {
        self.new_gps_state = true;
    }

    pub fn update(&mut self, state: &mut BlackboxStateMachine, writer: &mut SliceWriter, current_time_us: u32) -> usize {
        state.update(self, writer, current_time_us)
    }

    /// Called when the flight controller signals it has new data.
    #[allow(unused_results)]
    pub fn log_iteration(&mut self, current_time_us: u32, encoder: &mut SliceWriter) {
        // Write a keyframe every i_interval frames so we can resynchronise upon missing frames
        if self.should_log_i_frame() {
            // Don't log a slow frame if the slow data didn't change.
            // i_frames are already large enough without adding an additional item to write at the same time.
            // Unless we're *only* logging i_frames, then we have no choice.
            if self.is_only_logging_i_frames() && self.should_log_s_frame() {
                self.log_s_frame(encoder);
            }
            self.log_i_frame(encoder);
        } else {
            self.log_event_arming_beep_if_needed();
            self.log_event_flight_mode_if_needed(); // Check for FlightMode status change event

            if self.should_log_p_frame() {
                // ie p_frame_index == 0 && p_interval != 0
                // We assume that slow frames are only interesting in that they aid the interpretation of the main data stream.
                // So only log slow frames during loop iterations where we log a main frame.
                if self.should_log_p_frame() {
                    self.log_s_frame(encoder);
                }
                self.log_p_frame(encoder);
            }
            #[cfg(feature = "gps")]
            if Blackbox::field_enabled(self.log_select_enabled, LogFieldSelect::GPS) {
                if self.should_log_h_frame() {
                    self.home_latitude_degrees_1e7 = self.gps_state.home_latitude_degrees_1e7;
                    self.home_longitude_degrees_1e7 = self.gps_state.home_longitude_degrees_1e7;
                    self.home_altitude_cm = self.gps_state.home_altitude_cm;
                    let _len = self.log_h_frame(encoder);
                    let _len = self.log_g_frame(current_time_us, encoder);
                } else if self.should_log_g_frame() {
                    let _len = self.log_g_frame(current_time_us, encoder);
                }
            }
        }
    }

    pub fn should_log_i_frame(&self) -> bool {
        self.loop_index == 0
    }
    pub fn should_log_h_frame(&self) -> bool {
        self.features.is_set(Features::GPS)
    }
    pub fn should_log_g_frame(&self) -> bool {
        self.features.is_set(Features::GPS) && self.new_gps_state
    }
    pub fn should_log_p_frame(&self) -> bool {
        self.p_frame_index == 0 && self.p_interval != 0
    }
    /// If the data in the slow frame has changed, log a slow frame.
    ///
    /// The frame is also logged if it has been more than s_interval logging iterations
    /// since the field was last logged.
    // Write the slow frame periodically so it can be recovered if we ever lose sync
    pub fn should_log_s_frame(&self) -> bool {
        self.s_frame_index >= self.s_interval && self.new_slow_state
    }
    pub fn is_only_logging_i_frames(&self) -> bool {
        self.p_interval == 0
    }

    #[allow(clippy::unused_self)]
    pub fn log_event_arming_beep_if_needed(&self) {}
    #[allow(clippy::unused_self)]
    pub fn log_event_flight_mode_if_needed(&self) {} // Check for FlightMode status change event
}

impl Blackbox {
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
    //fn field_enabled(enabled_mask:u32, field:LogFieldSelect) -> bool { enabled_mask & (field as u32) }
    //pub fn is_field_enabled(&self, field:LogFieldSelect) ->bool { field_enabled(self.log_select_enabled, field) }
    // Helper function to check if a field is enabled
    pub fn field_enabled(enabled_mask: u32, field: u32) -> bool {
        enabled_mask & field != 0
    }

    // Public method to check if a log field is enabled
    pub fn is_field_enabled(&self, field: u32) -> bool {
        Self::field_enabled(self.log_select_enabled, field)
    }

    // Called from build_field_condition_cache(), which is called from start()
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

impl Blackbox {
    const MAIN_FIELDS: &[MainFieldDefinition] = crate::field_arrays::BLACKBOX_MAIN_FIELDS;

    pub fn send_header(writer: &mut SliceWriter) -> usize {
        writer.write_h_str("Product:Blackbox flight data recorder by Nicholas Sherlock\n");
        writer.write_h_str("Data version:2\n");
        writer.pos
    }

    pub fn send_main_field_header(&mut self, writer: &mut SliceWriter, index: usize) -> usize {
        let filter = |f: &MainFieldDefinition| self.conditions.test(f.condition);

        match index {
            0 => {
                // Name line. Note: This can exceed 500 bytes. Currently using buffer of size 1024, but perhaps this should be split.
                write_field_line(writer, 'I', "name", Self::MAIN_FIELDS.iter().filter(|&f| filter(f)), |w, f| {
                    w.write_str(f.name);
                    let index = f.field_name_index;
                    if index >= 0 {
                        w.write_char('[');
                        w.write_u8_ascii(index.cast_unsigned());
                        w.write_char(']');
                    }
                });
                writer.pos
            }
            1 => {
                // I Signed line
                let filtered = Self::MAIN_FIELDS.iter().filter(|&f| filter(f));
                write_field_line(writer, 'I', "signed", filtered, |w, f| {
                    w.write_u8_ascii(f.is_signed);
                });
                writer.pos
            }
            2 => {
                // I Predictor line
                let filtered = Self::MAIN_FIELDS.iter().filter(|&f| filter(f));
                write_field_line(writer, 'I', "predictor", filtered, |w, f| {
                    w.write_u8_ascii(f.i_predict);
                });
                writer.pos
            }
            3 => {
                // I Encoding line
                let filtered = Self::MAIN_FIELDS.iter().filter(|&f| filter(f));
                write_field_line(writer, 'I', "encoding", filtered, |w, f| {
                    w.write_u8_ascii(f.i_encode);
                });
                writer.pos
            }
            4 => {
                // P Predictor line
                let filtered = Self::MAIN_FIELDS.iter().filter(|&f| filter(f));
                write_field_line(writer, 'P', "predictor", filtered, |w, f| {
                    w.write_u8_ascii(f.p_predict);
                });
                writer.pos
            }
            5 => {
                // P Encoding line
                let filtered = Self::MAIN_FIELDS.iter().filter(|&f| filter(f));
                write_field_line(writer, 'P', "encoding", filtered, |w, f| {
                    w.write_u8_ascii(f.p_encode);
                });
                writer.pos
            }
            _ => 0,
        }
    }

    #[allow(clippy::unused_self)]
    pub fn send_slow_header(&mut self, writer: &mut SliceWriter) -> usize {
        let filter = |_: &SimpleFieldDefinition| true;

        // Name line.
        write_field_line(writer, 'S', "name", BLACKBOX_SLOW_FIELDS.iter().filter(|&f| filter(f)), |w, f| {
            w.write_str(f.name);
            let index = f.field_name_index;
            if index >= 0 {
                w.write_char('[');
                w.write_u8_ascii(index.cast_unsigned());
                w.write_char(']');
            }
        });
        // Signed line
        let filtered = BLACKBOX_SLOW_FIELDS.iter().filter(|&f| filter(f));
        write_field_line(writer, 'S', "signed", filtered, |w, f| {
            w.write_u8_ascii(f.is_signed);
        });
        // Predictor line
        let filtered = BLACKBOX_SLOW_FIELDS.iter().filter(|&f| filter(f));
        write_field_line(writer, 'S', "predictor", filtered, |w, f| {
            w.write_u8_ascii(f.predict);
        });
        // Encoding line
        let filtered = BLACKBOX_SLOW_FIELDS.iter().filter(|&f| filter(f));
        write_field_line(writer, 'S', "encoding", filtered, |w, f| {
            w.write_u8_ascii(f.encode);
        });
        writer.pos
    }

    pub fn send_sys_header(&mut self, writer: &mut SliceWriter, index: usize) -> usize {
        match index {
            0 => {
                writer.write_h_str("Firmware type:Cleanflight\n");
                writer.pos
            }
            1 => {
                writer.write_h_str("Firmware revision:Betaflight 3.3.1 (611bc70f8) REVOLT\n");
                writer.pos
            }
            2 => {
                writer.write_h_str("Firmware date:Mar 21 2018 00:00:00\n");

                writer.pos
            }
            3 => {
                writer.write_h_str("Log start datetime:0000-01-01T00:00:00.000+00:00\n");
                writer.pos
            }
            4 => {
                writer.write_h_str("Craft name:Protea\n");
                writer.pos
            }
            5 => {
                writer.write_h_str_u32_ascii("I interval:", self.i_interval);
                writer.pos
            }
            6 => {
                writer.write_h_str_u32_ascii("P interval:1/", self.p_interval);
                writer.pos
                // "P denom" ignored by blackbox-log-view
                // writer.write_h_str("P denom:32\n");
            }
            7 => {
                writer.write_h_str_u32_ascii("looptime:", self.looptime);
                writer.pos
            }
            8 => {
                writer.write_h_str("gyro_sync_denom:1\n");
                writer.write_h_str("pid_process_denom:1\n");

                writer.write_h_str("gyro_scale:0x3f800000\n");
                writer.write_h_str("acc_1G:4096\n");

                writer.write_h_str("features:541130760\n");
                writer.write_h_str("debug_mode:0\n");

                writer.write_h_str("minthrottle:1070\n");
                writer.write_h_str("maxthrottle:2000\n");
                writer.write_h_str("motorOutput:158,2047\n");

                writer.write_h_str("vbat_scale:110\n");
                writer.write_h_str("vbatcellvoltage:33,35,43\n");
                writer.write_h_str("vbatref:113\n");
                writer.write_h_str("currentSensor:0,235\n");
                writer.pos
            }
            _ => 0,
        }
        /*

        writer.pos*/
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(u8)]
pub enum BlackboxStateMachine {
    #[default]
    Disabled = 0,
    Stopped,
    PrepareLogFile,
    SendHeader,
    SendMainFieldHeader(usize),
    SendGpsHHeader,
    SendGpsGHeader,
    SendSlowHeader,
    SendSysinfo(usize),
    Paused,
    Running,
    ShuttingDown,
}

#[allow(dead_code)]
// Note: Not sure if this state machine is needed: it might naturally drop out of the embassy sync framework.
impl BlackboxStateMachine {
    pub fn start(&mut self, _start_params: BlackboxStartParameters) {
        *self = BlackboxStateMachine::PrepareLogFile;
    }

    pub fn finish(&mut self) {
        *self = BlackboxStateMachine::ShuttingDown;
    }

    pub fn set_state(&mut self, state: Self) {
        *self = state;
    }

    /// Called each flight loop iteration to perform blackbox logging.
    pub fn update(&mut self, ctx: &mut Blackbox, writer: &mut SliceWriter, current_time_us: u32) -> usize {
        #[allow(clippy::match_same_arms)]
        match core::mem::take(self) {
            BlackboxStateMachine::Disabled => {
                // If we are disabled, we stay disabled until start() is called
                // Explicitly setting *self = State::Disabled defends against a change in the default.
                *self = BlackboxStateMachine::Disabled;
                0
            }
            BlackboxStateMachine::Stopped => {
                *self = BlackboxStateMachine::Stopped;
                0
            }
            BlackboxStateMachine::PrepareLogFile => {
                ctx.logged_any_frames = false;
                *self = BlackboxStateMachine::SendHeader;
                0
            }
            BlackboxStateMachine::SendHeader => {
                *self = BlackboxStateMachine::SendMainFieldHeader(0);
                Blackbox::send_header(writer)
            }
            BlackboxStateMachine::SendMainFieldHeader(index) => {
                let len = ctx.send_main_field_header(writer, index);
                if len == 0 {
                    *self =
                        if ctx.features.is_set(Features::GPS) { BlackboxStateMachine::SendGpsHHeader } else { BlackboxStateMachine::SendSlowHeader }
                } else {
                    *self = BlackboxStateMachine::SendMainFieldHeader(index + 1);
                }
                len
            }
            BlackboxStateMachine::SendGpsHHeader => {
                *self = BlackboxStateMachine::SendGpsGHeader;
                0
            }
            BlackboxStateMachine::SendGpsGHeader => {
                *self = BlackboxStateMachine::SendSlowHeader;
                0
            }
            BlackboxStateMachine::SendSlowHeader => {
                *self = BlackboxStateMachine::SendSysinfo(0);
                ctx.send_slow_header(writer)
            }
            BlackboxStateMachine::SendSysinfo(index) => {
                let len = ctx.send_sys_header(writer, index);
                *self = if len == 0 { BlackboxStateMachine::Running } else { BlackboxStateMachine::SendSysinfo(index + 1) };
                len
            }
            BlackboxStateMachine::Paused => {
                *self = BlackboxStateMachine::Running;
                0
            }
            BlackboxStateMachine::Running => {
                //*self = State::Paused;
                ctx.log_iteration(current_time_us, writer);
                0
            }
            BlackboxStateMachine::ShuttingDown => {
                *self = BlackboxStateMachine::Stopped;
                0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    #![allow(unused_results)]
    #![allow(unused)]
    #![allow(clippy::unwrap_used)]
    use crate::{BlackboxTelemetry, sd_card::MockSdCard};

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
        let ctx = Blackbox::new();
        assert!(!ctx.logged_any_frames);
    }
    #[test]
    fn send_header() {
        let mut buffer = [0u8; 2048];
        //let mut sd_card = MockSdCard::new("state_machine_log.bbl");
        let pos = {
            let mut writer = SliceWriter { buffer: &mut buffer, pos: 0 };

            _ = Blackbox::send_header(&mut writer);

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
    fn send_main_field_header() {
        let mut buffer = [0u8; 2048];
        let mut writer = SliceWriter { buffer: &mut buffer, pos: 0 };
        let mut ctx = Blackbox::new();
        ctx.init(0);

        _ = ctx.send_main_field_header(&mut writer, 0);
        _ = ctx.send_main_field_header(&mut writer, 1);
        _ = ctx.send_main_field_header(&mut writer, 2);
        _ = ctx.send_main_field_header(&mut writer, 3);

        // Convert the written portion to a string for validation
        #[allow(clippy::unwrap_used)]
        let result = core::str::from_utf8(&writer.buffer[..writer.pos]).unwrap();
        // Print for manual inspection (if running with `cargo test -- --nocapture`)
        println!("\nMAIN FIELD HEADER\n{result}\n");
    }
    #[test]
    fn send_slow_header() {
        let mut buffer = [0u8; 2048];
        let mut writer = SliceWriter { buffer: &mut buffer, pos: 0 };
        let mut ctx = Blackbox::new();
        ctx.init(0);

        _ = ctx.send_slow_header(&mut writer);

        // Convert the written portion to a string for validation
        #[allow(clippy::unwrap_used)]
        let result = core::str::from_utf8(&writer.buffer[..writer.pos]).unwrap();
        // Print for manual inspection (if running with `cargo test -- --nocapture`)
        println!("\nSLOW HEADER\n{result}\n");
    }
    #[test]
    fn send_sys_header() {
        let mut buffer = [0u8; 2048];
        let mut writer = SliceWriter { buffer: &mut buffer, pos: 0 };
        let mut ctx = Blackbox::new();

        let mut index: usize = 0;
        loop {
            if ctx.send_sys_header(&mut writer, index) == 0 {
                break;
            }
            index += 1;
        }

        // Convert the written portion to a string for validation
        #[allow(clippy::unwrap_used)]
        let result = core::str::from_utf8(&writer.buffer[..writer.pos]).unwrap();
        // Print for manual inspection (if running with `cargo test -- --nocapture`)
        println!("\nSYS HEADER\n{result}\n");
    }
    #[test]
    fn state_machine() {
        println!("\nSTATE_MACHINE\n");
        let mut buffer = [0u8; 4096];
        let mut writer = SliceWriter { buffer: &mut buffer, pos: 0 };
        let mut ctx = Blackbox::new();
        //let mut _sd_card = MockSdCard::new("state_machine_log.bbl");
        ctx.init(0);

        let start = BlackboxStartParameters::new();
        let mut state = BlackboxStateMachine::default();
        let mut current_time_us: u32 = 0;
        let telemetry = BlackboxTelemetry::new();
        state.start(start);
        loop {
            ctx.load_telemetry(current_time_us, telemetry);
            _ = state.update(&mut ctx, &mut writer, current_time_us);
            //let state_i:u32 = state.into();
            //println!("state={state_i}");
            if state == BlackboxStateMachine::Running {
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
        ctx.init(0);

        let start = BlackboxStartParameters::new();
        let mut state = State::default();
        let mut current_time_us: u32 = 0;
        let telemetry = BlackboxTelemetry::new();
        state.start(start);
        let mut run_count = 0;
        loop {
            ctx.load_main_state(current_time_us, telemetry);
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
