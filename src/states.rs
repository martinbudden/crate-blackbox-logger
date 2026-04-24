#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlowState {
    pub flight_mode_flags: u32,
    pub state_flags: u8,
    pub failsafe_phase: u8,
    pub rx_signal_received: bool,
    pub rx_flight_channel_is_valid: bool,
}

impl Default for SlowState {
    fn default() -> Self {
        Self::new()
    }
}

impl SlowState {
    pub fn new() -> Self {
        Self {
            flight_mode_flags: 0,
            state_flags: 0,
            failsafe_phase: 0,
            rx_signal_received: false,
            rx_flight_channel_is_valid: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GpsPosition {
    pub longitude_degrees_1e7: i32, // longitude in degrees * 1e+7
    pub latitude_degrees_1e7: i32,  // latitude in degrees * 1e+7
    pub altitude_cm: i32,           // altitude in cm
}
impl Default for GpsPosition {
    fn default() -> Self {
        Self::new()
    }
}

impl GpsPosition {
    pub fn new() -> Self {
        Self { longitude_degrees_1e7: 0, latitude_degrees_1e7: 0, altitude_cm: 0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GpsState {
    pub time_of_week_ms: u32, // GPS time of week in ms
    pub interval_ms: u32,     // interval between GPS solutions in ms
    pub home: GpsPosition,    // home position
    pub position: GpsPosition,
    pub velocity_north_cmps: i16,        // north velocity, cm/s
    pub velocity_east_cmps: i16,         // east velocity, cm/s
    pub velocity_down_cmps: i16,         // down velocity, cm/s
    pub speed3d_cmps: i16,               // speed in cm/s
    pub ground_speed_cmps: i16,          // speed in cm/s
    pub ground_course_deci_degrees: i16, // Heading 2D in 10ths of a degree
    pub satellite_count: u8,
}

impl Default for GpsState {
    fn default() -> Self {
        Self::new()
    }
}

impl GpsState {
    pub fn new() -> Self {
        Self {
            time_of_week_ms: 0,
            interval_ms: 0,
            home: GpsPosition::default(),
            position: GpsPosition::default(),
            velocity_north_cmps: 0,
            velocity_east_cmps: 0,
            velocity_down_cmps: 0,
            speed3d_cmps: 0,
            ground_speed_cmps: 0,
            ground_course_deci_degrees: 0,
            satellite_count: 0,
        }
    }
}

impl GpsState {
    #[allow(unused)]
    pub fn state_changed(&self, new_state: Self) -> bool {
        //We could check for velocity changes as well but I doubt it changes independent of position
        new_state.satellite_count != self.satellite_count
            || new_state.position.latitude_degrees_1e7 != self.position.latitude_degrees_1e7
            || new_state.position.longitude_degrees_1e7 != self.position.longitude_degrees_1e7
    }
}

/// MainState is 152 bytes when all features enabled, so storing 3 copies for predictive purposes is not over onerous.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MainState {
    pub time_us: u32,
    pub baro_altitude: i32,
    #[cfg(feature = "rangefinder")]
    pub range_raw: i32,
    pub amperage: u16,
    pub battery_voltage: u16,
    pub rssi: u16,
    pub pid_p: [i32; Self::RPY_AXIS_COUNT],
    pub pid_i: [i32; Self::RPY_AXIS_COUNT],
    pub pid_d: [i32; Self::RPY_AXIS_COUNT],
    pub pid_s: [i32; Self::RPY_AXIS_COUNT],
    pub pid_k: [i32; Self::RPY_AXIS_COUNT],
    pub rc_commands: [i16; 4],
    pub setpoints: [i16; 4],
    pub gyro: [i16; Self::XYZ_AXIS_COUNT],
    pub gyro_unfiltered: [i16; Self::XYZ_AXIS_COUNT],
    pub acc: [i16; Self::XYZ_AXIS_COUNT],
    #[cfg(feature = "magnetometer")]
    pub mag: [i16; Self::XYZ_AXIS_COUNT],
    pub orientation: [i16; Self::XYZ_AXIS_COUNT], // only x,y,z from orientation quaternion are stored; w is always positive
    pub motor: [i16; Self::MAX_SUPPORTED_MOTOR_COUNT],
    pub erpm: [i16; Self::MAX_SUPPORTED_MOTOR_COUNT],
    pub debug: [i16; Self::DEBUG_VALUE_COUNT],
    #[cfg(feature = "servos")]
    pub servos: [i16; Self::MAX_SUPPORTED_SERVO_COUNT],
}

impl MainState {
    pub const RPY_AXIS_COUNT: usize = 3;
    pub const XYZ_AXIS_COUNT: usize = 3;
    pub const MAX_SUPPORTED_MOTOR_COUNT: usize = 8;
    #[cfg(feature = "servos")]
    pub const MAX_SUPPORTED_SERVO_COUNT: usize = 8;
    pub const DEBUG_VALUE_COUNT: usize = 4;
    pub const THROTTLE: usize = 3;
}

impl Default for MainState {
    fn default() -> Self {
        Self::new()
    }
}

impl MainState {
    pub fn new() -> Self {
        Self {
            time_us: 0,
            baro_altitude: 0,
            #[cfg(feature = "rangefinder")]
            range_raw: 0,
            amperage: 0,
            battery_voltage: 0,
            rssi: 0,
            pid_p: <[i32; Self::RPY_AXIS_COUNT]>::default(),
            pid_i: <[i32; Self::RPY_AXIS_COUNT]>::default(),
            pid_d: <[i32; Self::RPY_AXIS_COUNT]>::default(),
            pid_s: <[i32; Self::RPY_AXIS_COUNT]>::default(),
            pid_k: <[i32; Self::RPY_AXIS_COUNT]>::default(),
            rc_commands: <[i16; 4]>::default(),
            setpoints: <[i16; 4]>::default(),
            gyro: <[i16; Self::XYZ_AXIS_COUNT]>::default(),
            gyro_unfiltered: <[i16; Self::XYZ_AXIS_COUNT]>::default(),
            acc: <[i16; Self::XYZ_AXIS_COUNT]>::default(),
            #[cfg(feature = "magnetometer")]
            mag: <[i16; Self::XYZ_AXIS_COUNT]>::default(),
            orientation: <[i16; Self::XYZ_AXIS_COUNT]>::default(),
            motor: <[i16; Self::MAX_SUPPORTED_MOTOR_COUNT]>::default(),
            erpm: <[i16; Self::MAX_SUPPORTED_MOTOR_COUNT]>::default(),
            debug: <[i16; Self::DEBUG_VALUE_COUNT]>::default(),
            #[cfg(feature = "servos")]
            servos: <[i16; Self::MAX_SUPPORTED_SERVO_COUNT]>::default(),
        }
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
        is_full::<SlowState>();
        is_full::<GpsState>();
        is_full::<GpsPosition>();
        is_full::<MainState>();
    }
    #[test]
    fn new() {
        let slow_state = SlowState::new();
        assert_eq!(0, slow_state.flight_mode_flags);
        //assert_eq!(152, core::mem::size_of::<MainState>());

        let main_state = MainState::new();
        assert_eq!(0, main_state.time_us);

        let gps_state = GpsState::new();
        assert_eq!(0, gps_state.time_of_week_ms);
    }
}
