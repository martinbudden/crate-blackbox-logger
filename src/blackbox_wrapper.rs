use crate::blackbox::{Blackbox, BlackboxStateMachine};
use serde::{Deserialize, Serialize};

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
    pub fn new() -> Self {
        Self { debug_mode: 0, motor_count: 4, servo_count: 0 }
    }
}

#[allow(unused)]
#[derive(Clone, Copy, Debug)]
pub struct BlackboxWrapper {
    #[allow(dead_code)]
    state: BlackboxStateMachine,
    pub ctx: Blackbox,

    pub(crate) config: BlackboxConfig,
}

impl Default for BlackboxWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl BlackboxWrapper {
    pub fn new() -> Self {
        Self { state: BlackboxStateMachine::default(), ctx: Blackbox::default(), config: BlackboxConfig::default() }
    }
}

impl BlackboxWrapper {
#[allow(unused)]
    pub fn init(&mut self, config: BlackboxConfig) {
        //_serial_device.init();

        self.config = config;

        self.ctx.init(self.config.sample_rate);
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
        is_normal::<BlackboxWrapper>();
        is_full::<BlackboxStartParameters>();
        is_config::<BlackboxConfig>();
    }
    #[test]
    fn new() {
        let blackbox = BlackboxWrapper::default();
        assert_eq!(0, blackbox.ctx.iteration);
    }
}
