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
pub struct GpsState {
    pub time_of_week_ms: u32,            // GPS time of week in ms
    pub interval_ms: u32,                // interval between GPS solutions in ms
    pub home_longitude_degrees_1e7: i32, // home longitude in degrees * 1e+7
    pub home_latitude_degrees_1e7: i32,  // home latitude in degrees * 1e+7
    pub home_altitude_cm: i32,           // home altitude in cm
    pub longitude_degrees_1e7: i32,      // longitude in degrees * 1e+7
    pub latitude_degrees_1e7: i32,       // latitude in degrees * 1e+7
    pub altitude_cm: i32,                // altitude in cm
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
            home_longitude_degrees_1e7: 0,
            home_latitude_degrees_1e7: 0,
            home_altitude_cm: 0,
            longitude_degrees_1e7: 0,
            latitude_degrees_1e7: 0,
            altitude_cm: 0,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MainState {
    pub time_us: u32,
    pub baro_altitude: i32,
    pub surface_raw: i32,
    pub amperage_latest: i32,
    pub vbat_latest: i16,
    pub rssi: i16,
    pub axis_pid_p: [i16; Self::RPY_AXIS_COUNT],
    pub axis_pid_i: [i16; Self::RPY_AXIS_COUNT],
    pub axis_pid_d: [i16; Self::RPY_AXIS_COUNT],
    pub axis_pid_s: [i16; Self::RPY_AXIS_COUNT],
    pub axis_pid_k: [i16; Self::RPY_AXIS_COUNT],
    pub rc_commands: [i16; 4],
    pub setpoints: [i16; 4],
    pub gyro_adc: [i16; Self::XYZ_AXIS_COUNT],
    pub gyro_unfiltered: [i16; Self::XYZ_AXIS_COUNT],
    pub acc_adc: [i16; Self::XYZ_AXIS_COUNT],
    pub orientation: [i16; Self::XYZ_AXIS_COUNT], // only x,y,z from orientation quaternion are stored; w is always positive
    pub motors: [i16; Self::MAX_SUPPORTED_MOTOR_COUNT],
    pub erpms: [i16; Self::MAX_SUPPORTED_MOTOR_COUNT],
    pub debug: [i16; Self::DEBUG_VALUE_COUNT],

    #[cfg(feature = "servos")]
    pub servos: [i16; Self::MAX_SUPPORTED_SERVO_COUNT],
}

impl MainState {
    const RPY_AXIS_COUNT: usize = 3;
    const XYZ_AXIS_COUNT: usize = 3;
    pub const MAX_SUPPORTED_MOTOR_COUNT: usize = 8;
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
            surface_raw: 0,
            amperage_latest: 0,
            vbat_latest: 0,
            rssi: 0,
            axis_pid_p: <[i16; Self::RPY_AXIS_COUNT]>::default(),
            axis_pid_i: <[i16; Self::RPY_AXIS_COUNT]>::default(),
            axis_pid_d: <[i16; Self::RPY_AXIS_COUNT]>::default(),
            axis_pid_s: <[i16; Self::RPY_AXIS_COUNT]>::default(),
            axis_pid_k: <[i16; Self::RPY_AXIS_COUNT]>::default(),
            rc_commands: <[i16; 4]>::default(),
            setpoints: <[i16; 4]>::default(),
            gyro_adc: <[i16; Self::XYZ_AXIS_COUNT]>::default(),
            gyro_unfiltered: <[i16; Self::XYZ_AXIS_COUNT]>::default(),
            acc_adc: <[i16; Self::XYZ_AXIS_COUNT]>::default(),
            orientation: <[i16; Self::XYZ_AXIS_COUNT]>::default(),
            motors: <[i16; Self::MAX_SUPPORTED_MOTOR_COUNT]>::default(),
            erpms: <[i16; Self::MAX_SUPPORTED_MOTOR_COUNT]>::default(),
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
        is_full::<MainState>();
    }
    #[test]
    fn new() {
        let slow_state = SlowState::new();
        assert_eq!(0, slow_state.flight_mode_flags);

        let main_state = MainState::new();
        assert_eq!(0, main_state.time_us);

        let gps_state = GpsState::new();
        assert_eq!(0, gps_state.time_of_week_ms);
    }
}
