#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Features {
    pub(crate) flags: u32,
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
    pub const THREE_D: u32 = 1 << 12;
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

#[cfg(test)]
mod tests {
    #[allow(unused)]
    use super::*;

    #[allow(unused)]
    fn is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<Features>();
    }
    #[test]
    fn new() {
        let features = Features::default();
        assert!(!features.is_set(Features::GPS));
    }
}
