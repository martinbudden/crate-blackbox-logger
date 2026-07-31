#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlackboxEventId {}
impl BlackboxEventId {
    pub const SYNC_BEEP: u8 = 0;
    #[allow(unused)]
    pub const AUTOTUNE_CYCLE_START: u8 = 10;
    #[allow(unused)]
    pub const AUTOTUNE_CYCLE_RESULT: u8 = 11;
    #[allow(unused)]
    pub const AUTOTUNE_TARGETS: u8 = 12;
    pub const INFLIGHT_ADJUSTMENT: u8 = 13;
    pub const LOGGING_RESUME: u8 = 14;
    pub const DISARM: u8 = 15;
    pub const FLIGHT_MODE: u8 = 30;
    pub const LOG_END: u8 = 255;
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum BlackboxEvent {
    SyncBeep(u32) = 0,
    AutotuneCycleStart = 10,
    AutotuneCycleResult = 11,
    AutotuneCycleTargets = 12,
    InflightAdjustment(i32, f32, u8, bool) = 13,
    LoggingResume(u32, u32) = 14,
    Disarm(u32) = 15,
    FlightMode(u32, u32) = 30,
    LogEnd = 255,
}

impl Default for BlackboxEvent {
    fn default() -> Self {
        Self::new()
    }
}

impl BlackboxEvent {
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self::SyncBeep(0)
    }
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlackboxSlowData {
    pub flight_mode_flags: u32,
    pub gps_state_flags: u8,
    pub failsafe_phase: u8,
    pub rx_signal_received: bool,
    pub rx_flight_channel_is_valid: bool,
}

impl Default for BlackboxSlowData {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(missing_docs)]
#[allow(unused)]
impl BlackboxSlowData {
    pub const FLIGHT_MODE_BLACKBOX_ACTIVE: u32 = 1 << 23;

    pub const GPS_FIX_HOME: u8 = 0x01;
    pub const GPS_STATE_FIX: u8 = 0x02;
    pub const GPS_STATE_FIX_EVER: u8 = 0x04;

    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            flight_mode_flags: 0,
            gps_state_flags: 0,
            failsafe_phase: 0,
            rx_signal_received: false,
            rx_flight_channel_is_valid: false,
        }
    }

    #[must_use]
    pub fn is_blackbox_active(&self) -> bool {
        self.flight_mode_flags & Self::FLIGHT_MODE_BLACKBOX_ACTIVE != 0
    }
}

/// GPS position as longitude, latitude, and altitude.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlackboxGpsPosition {
    /// longitude in degrees * 1e+7.
    pub longitude_degrees_1e7: i32,
    /// latitude in degrees * 1e+7.
    pub latitude_degrees_1e7: i32,
    /// altitude in cm.
    pub altitude_cm: i32,
}

impl Default for BlackboxGpsPosition {
    fn default() -> Self {
        Self::new()
    }
}

impl BlackboxGpsPosition {
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self { longitude_degrees_1e7: 0, latitude_degrees_1e7: 0, altitude_cm: 0 }
    }
}


/// GPS data that is recorded.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlackboxGpsData {
    /// GPS time of week in ms.
    pub time_of_week_ms: u32,
    /// Interval between GPS solutions in ms.
    pub interval_ms: u32,
    /// Current position.
    pub position: BlackboxGpsPosition,
    /// North velocity, cm/s.
    pub velocity_north_cmps: i16,
    /// East velocity, cm/s.
    pub velocity_east_cmps: i16,
    /// Down velocity, cm/s.
    pub velocity_down_cmps: i16,
    /// Speed in cm/s.
    pub speed3d_cmps: i16,
    /// Speed in cm/s.
    pub ground_speed_cmps: i16,
    /// Heading 2D in 10ths of a degree.
    pub ground_course_degrees_x10: i16,
    /// Number of satellites used for GPS fix.
    pub satellite_count: u8,
}

impl Default for BlackboxGpsData {
    fn default() -> Self {
        Self::new()
    }
}

impl BlackboxGpsData {
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            time_of_week_ms: 0,
            interval_ms: 0,
            position: BlackboxGpsPosition::new(),
            velocity_north_cmps: 0,
            velocity_east_cmps: 0,
            velocity_down_cmps: 0,
            speed3d_cmps: 0,
            ground_speed_cmps: 0,
            ground_course_degrees_x10: 0,
            satellite_count: 0,
        }
    }
}

impl BlackboxGpsData {
    /// Returns true if GPS data is different from previous data.
    #[must_use]
    pub fn state_changed(&self, new_data: Self) -> bool {
        // We could check for velocity changes as well but I doubt it changes independent of position
        new_data.satellite_count != self.satellite_count
            || new_data.position.latitude_degrees_1e7 != self.position.latitude_degrees_1e7
            || new_data.position.longitude_degrees_1e7 != self.position.longitude_degrees_1e7
    }
}

/// `MainData` is about 150 bytes when all features enabled, so storing 3 copies for predictive purposes is not over onerous.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlackboxMainData {
    pub time_us: u32,
    pub pid_p: [i32; Self::RPY_AXIS_COUNT],
    pub pid_i: [i32; Self::RPY_AXIS_COUNT],
    pub pid_d: [i32; Self::RPY_AXIS_COUNT],
    pub pid_s: [i32; Self::RPY_AXIS_COUNT],
    pub pid_k: [i32; Self::RPY_AXIS_COUNT],
    pub rc_commands: [u16; 4],
    pub setpoints: [i16; Self::SETPOINT_COUNT],
    pub gyro: [i16; Self::XYZ_AXIS_COUNT],
    pub gyro_unfiltered: [i16; Self::XYZ_AXIS_COUNT],
    pub acc: [i16; Self::XYZ_AXIS_COUNT],
    #[cfg(feature = "magnetometer")]
    pub mag: [i16; Self::XYZ_AXIS_COUNT],
    /// only x,y,z from orientation quaternion are stored; w is always positive.
    pub orientation: [i16; Self::XYZ_AXIS_COUNT],
    pub motor: [u16; Self::MAX_SUPPORTED_MOTOR_COUNT],
    #[cfg(feature = "dshot_telemetry")]
    pub erpm: [u16; Self::MAX_SUPPORTED_MOTOR_COUNT],
    pub debug: [i16; Self::DEBUG_COUNT],
    #[cfg(feature = "servos")]
    pub servos: [i16; Self::MAX_SUPPORTED_SERVO_COUNT],
    pub baro_altitude: i32,
    pub range_raw: i32,
    pub amperage: i16,
    pub battery_voltage: u16,
    pub rssi: u16,
}

#[allow(missing_docs)]
impl BlackboxMainData {
    pub const RPY_AXIS_COUNT: usize = 3;
    pub const XYZ_AXIS_COUNT: usize = 3;
    #[cfg(feature = "eight_motors")]
    pub const MAX_SUPPORTED_MOTOR_COUNT: usize = 8;
    #[cfg(not(feature = "eight_motors"))]
    pub const MAX_SUPPORTED_MOTOR_COUNT: usize = 4;
    #[cfg(feature = "servos")]
    pub const MAX_SUPPORTED_SERVO_COUNT: usize = 8;
    pub const DEBUG_COUNT: usize = 8;
    pub const SETPOINT_COUNT: usize = 4;
}

impl Default for BlackboxMainData {
    fn default() -> Self {
        Self::new()
    }
}

impl BlackboxMainData {
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            time_us: 0,
            pid_p: [0i32; Self::RPY_AXIS_COUNT],
            pid_i: [0i32; Self::RPY_AXIS_COUNT],
            pid_d: [0i32; Self::RPY_AXIS_COUNT],
            pid_s: [0i32; Self::RPY_AXIS_COUNT],
            pid_k: [0i32; Self::RPY_AXIS_COUNT],
            rc_commands: [1500, 1500, 1500, 1000],
            setpoints: [0i16; Self::SETPOINT_COUNT],
            gyro: [0i16; Self::XYZ_AXIS_COUNT],
            gyro_unfiltered: [0i16; Self::XYZ_AXIS_COUNT],
            acc: [0i16; Self::XYZ_AXIS_COUNT],
            #[cfg(feature = "magnetometer")]
            mag: [0i16; Self::XYZ_AXIS_COUNT],

            orientation: [0i16; Self::XYZ_AXIS_COUNT],
            #[cfg(feature = "eight_motors")]
            motor: [1100, 1100, 1100, 1100, 1100, 1100, 1100, 1100],
            #[cfg(not(feature = "eight_motors"))]
            motor: [1100, 1100, 1100, 1100],

            #[cfg(feature = "dshot_telemetry")]
            erpm: [0u16; Self::MAX_SUPPORTED_MOTOR_COUNT],
            debug: [0i16; Self::DEBUG_COUNT],
            #[cfg(feature = "servos")]
            servos: [0i16; Self::MAX_SUPPORTED_SERVO_COUNT],
            baro_altitude: 0,
            range_raw: 0,
            amperage: 0,
            battery_voltage: 0,
            rssi: 0,
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
        is_full::<BlackboxSlowData>();
        is_full::<BlackboxGpsData>();
        is_full::<BlackboxGpsPosition>();
        is_full::<BlackboxMainData>();
    }
    #[test]
    fn new() {
        let slow_data = BlackboxSlowData::new();
        assert_eq!(0, slow_data.flight_mode_flags);
        //assert_eq!(152, core::mem::size_of::<MainState>());

        let main_data = BlackboxMainData::new();
        assert_eq!(0, main_data.time_us);

        let gps_data = BlackboxGpsData::new();
        assert_eq!(0, gps_data.time_of_week_ms);
    }
}
