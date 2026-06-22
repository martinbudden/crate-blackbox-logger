use crate::SliceEncoder;
use crate::logger::Logger;
use crate::logger_state::LoggerState;
use crate::{GyroPidMessage, SetpointMessage};
#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

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

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BlackboxConfig {
    pub sample_rate: u8,
    pub device: u8,
    pub mode: u8,
    pub gps_use_3d_speed: bool,
    pub fields_disabled_mask: u32,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for BlackboxConfig {}

impl Default for BlackboxConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl BlackboxConfig {
    #[must_use]
    pub const fn new() -> Self {
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
pub struct BlackboxStartParameters {
    pub debug_mode: u16,
    pub motor_count: u8,
    pub servo_count: u8,
}

impl Default for BlackboxStartParameters {
    fn default() -> Self {
        Self::new()
    }
}

impl BlackboxStartParameters {
    #[must_use]
    pub const fn new() -> Self {
        Self { debug_mode: 0, motor_count: 4, servo_count: 0 }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Blackbox {
    state: LoggerState,
    pub logger: Logger,

    config: BlackboxConfig,
}

impl Default for Blackbox {
    fn default() -> Self {
        Self::new(BlackboxConfig::new(), 0)
    }
}

impl Blackbox {
    #[must_use]
    pub const fn new(config: BlackboxConfig, features: u32) -> Self {
        Self { state: LoggerState::new(), logger: Logger::new(features), config }
    }
}

impl Blackbox {
    pub fn init(&mut self) {
        //_serial_device.init();
        self.logger.init(self.config.sample_rate, self.config.fields_disabled_mask);
    }
}

impl Blackbox {
    pub fn load_telemetry(&mut self, current_time_us: u32, gyro_pid: GyroPidMessage, setpoint: SetpointMessage) {
        self.logger.load_telemetry(current_time_us, gyro_pid, setpoint);
    }
    pub fn load_slow_telemetry(&mut self, setpoint: SetpointMessage) {
        self.logger.load_slow_telemetry(setpoint);
    }
    pub fn update(&mut self, encoder: &mut SliceEncoder, current_time_us: u32, is_active: bool) -> usize {
        self.state.update(&mut self.logger, encoder, current_time_us, is_active)
    }
    pub fn set_state(&mut self, state: LoggerState) {
        self.state.set_state(state);
    }
    #[must_use]
    pub fn state(&self) -> LoggerState {
        self.state
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_config<T: Serialize + for<'a> Deserialize<'a> + for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_normal::<Blackbox>();
        is_full::<BlackboxStartParameters>();
        is_full::<BlackboxConfig>();
        #[cfg(feature = "serde")]
        is_config::<BlackboxConfig>();
    }
    #[test]
    fn test_new() {
        let blackbox = Blackbox::default();
        assert_eq!(0, blackbox.logger.iteration);
    }
}
