use super::{
    BlackboxConfig, SliceEncoder,
    data::{BlackboxMainData, BlackboxSlowData},
    logger::{BlackboxSysInfo, Logger},
    logger_state::LoggerState,
};

#[cfg(feature = "gps")]
use super::BlackboxGpsData;

/// Blackbox struct containing the logger, state machine, and config.
#[derive(Clone, Copy, Debug)]
pub struct Blackbox {
    state: LoggerState,
    logger: Logger,
    config: BlackboxConfig,
}

impl Default for Blackbox {
    fn default() -> Self {
        Self::new(BlackboxConfig::new(), BlackboxSysInfo::new())
    }
}

impl Blackbox {
    /// Constructor.
    #[must_use]
    pub const fn new(config: BlackboxConfig, sys_info: BlackboxSysInfo) -> Self {
        Self { state: LoggerState::new(), logger: Logger::new(sys_info), config }
    }
}

#[allow(missing_docs)]
impl Blackbox {
    pub fn init(&mut self) {
        //_serial_device.init();
        self.logger.init(self.config.sample_rate, self.config.fields_disabled_mask, self.config.huffman_compress);
    }

    /*pub fn load_telemetry(&mut self, current_time_us: u64, gyro_pid: GyroPidMessage, setpoint: SetpointMessage) {
        self.logger.load_telemetry(current_time_us, gyro_pid, setpoint);
    }*/

    #[inline]
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.logger.slow_data.is_blackbox_active()
    }

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
    #[must_use]
    pub fn i_interval(&self) -> u32 {
        self.logger.i_interval
    }

    #[inline]
    #[must_use]
    pub fn p_interval(&self) -> u32 {
        self.logger.p_interval
    }

    #[inline]
    pub fn start(&mut self, debug_mode: u16) {
        self.state.start(debug_mode);
    }

    #[inline]
    pub fn update(&mut self, encoder: &mut SliceEncoder, current_time_us: u64, force_i_frame: bool) -> usize {
        self.state.update(&mut self.logger, encoder, current_time_us, force_i_frame)
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
mod test_traits {
    use super::*;

    fn is_normal<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    fn normal_types() {
        is_normal::<Blackbox>();
    }
}

mod tests {
    #[allow(unused)]
    use super::*;

    #[test]
    fn test_new() {
        let blackbox = Blackbox::default();
        assert_eq!(0, blackbox.logger.iteration);
    }
}
