use crate::BlackboxTelemetry;
use crate::GpsState;
use crate::LogFieldSelect;
use crate::SliceWriter;
use crate::blackbox_context::{BlackboxContext, State};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Features {
    flags: u32,
}
impl Features {
    pub const RX_PPM: u32 = 1 << 0;
    pub const VBAT: u32 = 1 << 1;
    pub const INFLIGHT_ACC_CAL: u32 = 1 << 2;
    pub const RX_SERIAL: u32 = 1 << 3;
    pub const MOTOR_STOP: u32 = 1 << 4;
    pub const SERVO_TILT: u32 = 1 << 5;
    pub const SOFT_SERIAL: u32 = 1 << 6;
    pub const GPS: u32 = 1 << 7;
    pub const FAILSAFE: u32 = 1 << 8;
    pub const SONAR: u32 = 1 << 9;
    pub const TELEMETRY: u32 = 1 << 10;
    pub const CURRENT_METER: u32 = 1 << 11;
    //pub const 3D:u32 = 1 << 12;
    pub const RX_PARALLEL_PWM: u32 = 1 << 13;
    pub const RX_MSP: u32 = 1 << 14;
    pub const RSSI_ADC: u32 = 1 << 15;
    pub const LED_STRIP: u32 = 1 << 16;
    pub const DISPLAY: u32 = 1 << 17;
    pub const ONESHOT125: u32 = 1 << 18;
    pub const BLACKBOX: u32 = 1 << 19;
    pub const CHANNEL_FORWARDING: u32 = 1 << 20;
    pub const TRANSPONDER: u32 = 1 << 21;

    pub fn set(&mut self, flag: u32) {
        self.flags |= flag;
    }
    pub fn is_set(&self, flag: u32) -> bool {
        self.flags & flag != 0
    }
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
    #[allow(dead_code)]
    state: State,
    #[allow(dead_code)]
    pub ctx: BlackboxContext,

    pub(crate) config: BlackboxConfig,
    pub buf: [u8; 1024],
}

impl Default for Blackbox {
    fn default() -> Self {
        Self::new()
    }
}

impl Blackbox {
    pub fn new() -> Self {
        Self {
            state: State::default(),
            ctx: BlackboxContext::default(),
            config: BlackboxConfig::default(),
            buf: [0u8; 1024],
        }
    }
}

impl Blackbox {
    pub fn init(&mut self, config: BlackboxConfig) {
        //_serial_device.init();

        self.config = config;

        self.ctx.init(self.config.sample_rate);
    }
}

impl BlackboxContext {
    pub fn load_main_state(&mut self, current_time_us: u32, telemetry: BlackboxTelemetry) {
        let current = &mut self.main_states[self.main_state_index_current];
        current.time_us = current_time_us;
        current.acc = (telemetry.acc * 4096.0).into();
        current.gyro = (telemetry.gyro_rps.to_degrees()).into();
        current.gyro_unfiltered = (telemetry.gyro_rps_unfiltered.to_degrees()).into();
    }
    // Callbacks are probably better done with Generics, but this is simpler for now.
    // TODO: Actually, on second thought, it is probably better if this just peeks at the relevant watchers.
    /// Called when the flight controller signals it has new data.
    pub fn log_iteration(&mut self, current_time_us: u32, telemetry: BlackboxTelemetry, encoder: &mut SliceWriter) {
        // Write a keyframe every i_interval frames so we can resynchronise upon missing frames
        if self.should_log_i_frame() {
            // ie _loop_index == 0
            // Don't log a slow frame if the slow data didn't change (i_frames are already large enough without adding
            // an additional item to write at the same time). Unless we're *only* logging i_frames, then we have no choice.
            if self.is_only_logging_i_frames() {
                let _len = self.log_s_frame_if_needed(encoder);
                //self.sd_card.write_all(&self.buf[..len]).await.ok();
            }

            self.load_main_state(current_time_us, telemetry);
            let _len = self.log_i_frame(encoder);
            //self.sd_card.write_all(&self.buf[..len]).await.ok();
        } else {
            self.log_event_arming_beep_if_needed();
            self.log_event_flight_mode_if_needed(); // Check for FlightMode status change event

            if self.should_log_p_frame() {
                // ie p_frame_index == 0 && p_interval != 0
                // We assume that slow frames are only interesting in that they aid the interpretation of the main data stream.
                // So only log slow frames during loop iterations where we log a main frame.
                let _len = self.log_s_frame_if_needed(encoder);
                //self.sd_card.write_all(&self.buf[..len]).await.ok();

                self.load_main_state(current_time_us, telemetry);
                let _len = self.log_p_frame(encoder);
                //self.sd_card.write_all(&self.buf[..len]).await.ok();
            }
            #[cfg(feature = "gps")]
            if BlackboxContext::field_enabled(self.log_select_enabled, LogFieldSelect::GPS) {
                let gps_state_new = GpsState::new();

                let gps_state_changed = gps_state_new.satellite_count != self.gps_state.satellite_count
                    || gps_state_new.latitude_degrees_1e7 != self.gps_state.latitude_degrees_1e7
                    || gps_state_new.longitude_degrees_1e7 != self.gps_state.longitude_degrees_1e7;

                self.gps_state = gps_state_new;

                if self.should_log_h_frame() {
                    self.home_latitude_degrees_1e7 = self.gps_state.home_latitude_degrees_1e7;
                    self.home_longitude_degrees_1e7 = self.gps_state.home_longitude_degrees_1e7;
                    self.home_altitude_cm = self.gps_state.home_altitude_cm;
                    let _len = self.log_h_frame(encoder);
                    //self.sd_card.write_all(&self.buf[..len]).await.ok();
                    let _len = self.log_g_frame(current_time_us, encoder);
                //self.sd_card.write_all(&self.buf[..len]).await.ok();
                } else if gps_state_changed {
                    //We could check for velocity changes as well but I doubt it changes independent of position
                    let _len = self.log_g_frame(current_time_us, encoder);
                    //self.sd_card.write_all(&self.buf[..len]).await.ok();
                }
            }
        }
    }

    pub fn should_log_i_frame(&self) -> bool {
        self.loop_index == 0
    }
    #[allow(clippy::unused_self)]
    pub fn should_log_h_frame(&self) -> bool {
        true
    }
    pub fn should_log_p_frame(&self) -> bool {
        self.p_frame_index == 0 && self.p_interval != 0
    }
    pub fn is_only_logging_i_frames(&self) -> bool {
        self.p_interval == 0
    }

    #[allow(clippy::unused_self)]
    pub fn log_event_arming_beep_if_needed(&self) {}
    #[allow(clippy::unused_self)]
    pub fn log_event_flight_mode_if_needed(&self) {} // Check for FlightMode status change event

    /// If the data in the slow frame has changed, log a slow frame.
    ///
    /// The frame is also logged if it has been more than s_interval logging iterations
    /// since the field was last logged.
    pub fn log_s_frame_if_needed(&mut self, encoder: &mut SliceWriter) -> usize {
        // Write the slow frame periodically so it can be recovered if we ever lose sync
        // TODO: add a check if new slow data has arrived
        if self.s_frame_index >= self.s_interval {
            return self.log_s_frame(encoder);
        }
        0
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
        assert_eq!(0, blackbox.ctx.iteration);
    }
}
