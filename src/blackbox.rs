use crate::{
    SliceEncoder,
    data::{BlackboxMainData, BlackboxSlowData},
    logger::Logger,
    logger_state::LoggerState,
};

#[cfg(feature = "gps")]
use crate::BlackboxGpsData;

#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(u8)]
pub enum BlackboxDevice {
    #[default]
    None,
    Flash,
    SdCard,
    Serial,
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(u8)]
pub enum BlackboxMode {
    #[default]
    Normal,
    MotorTest,
    AlwaysOne,
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BlackboxConfig {
    pub fields_disabled_mask: u32,
    pub sample_rate: u8,
    pub device: u8,
    pub mode: u8,
    pub high_resolution: u8,
    pub gps_use_3d_speed: bool,
    pub huffman_compress: bool,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for BlackboxConfig {}

impl Default for BlackboxConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl BlackboxConfig {
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            fields_disabled_mask: 0,
            sample_rate: 0,
            device: BlackboxDevice::None as u8,
            mode: BlackboxMode::Normal as u8,
            high_resolution: 0,
            gps_use_3d_speed: false,
            huffman_compress: false,
        }
    }
}
#[allow(missing_docs)]
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
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self { debug_mode: 0, motor_count: 4, servo_count: 0 }
    }
}

/// Blackbox struct containing the logger, state machine, and config.
#[derive(Clone, Copy, Debug)]
pub struct Blackbox {
    state: LoggerState,
    logger: Logger,

    config: BlackboxConfig,
}

impl Default for Blackbox {
    fn default() -> Self {
        Self::new(BlackboxConfig::new())
    }
}

impl Blackbox {
    /// Constructor.
    #[must_use]
    pub const fn new(config: BlackboxConfig) -> Self {
        Self { state: LoggerState::new(), logger: Logger::new(), config }
    }
}

#[allow(missing_docs)]
impl Blackbox {
    pub fn init(&mut self) {
        //_serial_device.init();
        self.logger.init(self.config.sample_rate, self.config.fields_disabled_mask, self.config.huffman_compress);
    }

    /*pub fn load_telemetry(&mut self, current_time_us: u32, gyro_pid: GyroPidMessage, setpoint: SetpointMessage) {
        self.logger.load_telemetry(current_time_us, gyro_pid, setpoint);
    }*/

    #[inline]
    pub fn set_main_data(&mut self, main_data: BlackboxMainData) {
        self.logger.set_main_data(main_data);
    }

    #[inline]
    pub fn set_slow_data(&mut self, slow_data: BlackboxSlowData) {
        self.logger.set_slow_data(slow_data);
    }

    #[inline]
    #[cfg(feature = "gps")]
    pub fn set_gps_data(&mut self, gps_data: BlackboxGpsData) {
        self.logger.set_gps_data(gps_data);
    }

    #[inline]
    pub fn update(&mut self, encoder: &mut SliceEncoder, current_time_us: u32) -> usize {
        self.state.update(&mut self.logger, encoder, current_time_us)
    }

    #[inline]
    pub fn set_state(&mut self, state: LoggerState) {
        self.state.set_state(state);
    }

    #[inline]
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
