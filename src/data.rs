use crate::{BlackboxGpsTelemetry, BlackboxSlowTelemetry, BlackboxTelemetry};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Event {}
impl Event {
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
    pub const FLIGHT_MODE: u8 = 30; // Add new event type for flight mode status.
    pub const LOG_END: u8 = 255;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlowData {
    pub flight_mode_flags: u32,
    pub state_flags: u8,
    pub failsafe_phase: u8,
    pub rx_signal_received: bool,
    pub rx_flight_channel_is_valid: bool,
    pub debug: [i16; Self::SLOW_DEBUG_COUNT],
}

impl SlowData {
    pub const SLOW_DEBUG_COUNT: usize = BlackboxSlowTelemetry::SLOW_DEBUG_COUNT;
}

impl Default for SlowData {
    fn default() -> Self {
        Self::new()
    }
}

impl SlowData {
    pub fn new() -> Self {
        Self {
            flight_mode_flags: 0,
            state_flags: 0,
            failsafe_phase: 0,
            rx_signal_received: false,
            rx_flight_channel_is_valid: false,
            debug: <[i16; Self::SLOW_DEBUG_COUNT]>::default(),
        }
    }
}

impl From<BlackboxSlowTelemetry> for SlowData {
    fn from(telemetry: BlackboxSlowTelemetry) -> Self {
        Self {
            flight_mode_flags: telemetry.flight_mode_flags,
            state_flags: telemetry.state_flags,
            failsafe_phase: telemetry.failsafe_phase,
            rx_signal_received: telemetry.rx_signal_received,
            rx_flight_channel_is_valid: telemetry.rx_flight_channel_is_valid,
            debug: telemetry.debug,
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
pub struct GpsData {
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

impl Default for GpsData {
    fn default() -> Self {
        Self::new()
    }
}

impl GpsData {
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

impl From<BlackboxGpsTelemetry> for GpsData {
    fn from(_telemetry: BlackboxGpsTelemetry) -> Self {
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

impl GpsData {
    #[allow(unused)]
    pub fn state_changed(&self, new_data: Self) -> bool {
        //We could check for velocity changes as well but I doubt it changes independent of position
        new_data.satellite_count != self.satellite_count
            || new_data.position.latitude_degrees_1e7 != self.position.latitude_degrees_1e7
            || new_data.position.longitude_degrees_1e7 != self.position.longitude_degrees_1e7
    }
}

/// MainData is about 150 bytes when all features enabled, so storing 3 copies for predictive purposes is not over onerous.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MainData {
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
    #[cfg(feature = "dshot_telemetry")]
    pub erpm: [i16; Self::MAX_SUPPORTED_MOTOR_COUNT],
    pub debug: [i16; Self::DEBUG_COUNT],
    #[cfg(feature = "servos")]
    pub servos: [i16; Self::MAX_SUPPORTED_SERVO_COUNT],
}

impl MainData {
    pub const RPY_AXIS_COUNT: usize = 3;
    #[allow(unused)]
    pub const RP_AXIS_COUNT: usize = 2;
    pub const XYZ_AXIS_COUNT: usize = 3;
    pub const RC_COMMAND_COUNT: usize = BlackboxTelemetry::RC_COMMAND_COUNT;
    pub const MAX_SUPPORTED_MOTOR_COUNT: usize = BlackboxTelemetry::MAX_SUPPORTED_MOTOR_COUNT;
    #[cfg(feature = "servos")]
    pub const MAX_SUPPORTED_SERVO_COUNT: usize = BlackboxTelemetry::MAX_SUPPORTED_SERVO_COUNT;
    pub const DEBUG_COUNT: usize = 8;
    pub const SETPOINT_COUNT: usize = BlackboxTelemetry::SETPOINT_COUNT;
    //pub const PID_ERROR_COUNT: usize = BlackboxTelemetry::PID_ERROR_COUNT;
    pub const THROTTLE: usize = 3;
}

impl Default for MainData {
    fn default() -> Self {
        Self::new()
    }
}

impl MainData {
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
            rc_commands: <[i16; Self::RC_COMMAND_COUNT]>::default(),
            setpoints: <[i16; Self::SETPOINT_COUNT]>::default(),
            gyro: <[i16; Self::XYZ_AXIS_COUNT]>::default(),
            gyro_unfiltered: <[i16; Self::XYZ_AXIS_COUNT]>::default(),
            acc: <[i16; Self::XYZ_AXIS_COUNT]>::default(),
            #[cfg(feature = "magnetometer")]
            mag: <[i16; Self::XYZ_AXIS_COUNT]>::default(),
            orientation: <[i16; Self::XYZ_AXIS_COUNT]>::default(),
            motor: <[i16; Self::MAX_SUPPORTED_MOTOR_COUNT]>::default(),
            #[cfg(feature = "dshot_telemetry")]
            erpm: <[i16; Self::MAX_SUPPORTED_MOTOR_COUNT]>::default(),
            debug: <[i16; Self::DEBUG_COUNT]>::default(),
            #[cfg(feature = "servos")]
            servos: <[i16; Self::MAX_SUPPORTED_SERVO_COUNT]>::default(),
        }
    }
}

impl From<BlackboxTelemetry> for MainData {
    fn from(telemetry: BlackboxTelemetry) -> Self {
        const TO_I16: f32 = 32_757.0;
        Self {
            time_us: telemetry.time_us,
            baro_altitude: 0,
            #[cfg(feature = "rangefinder")]
            range_raw: 0,
            amperage: 0,
            battery_voltage: 0,
            rssi: 0,
            pid_p: [
                i32::from(telemetry.pid_errors_p[0]),
                i32::from(telemetry.pid_errors_p[1]),
                i32::from(telemetry.pid_errors_p[2]),
            ],
            pid_i: [
                i32::from(telemetry.pid_errors_i[0]),
                i32::from(telemetry.pid_errors_i[1]),
                i32::from(telemetry.pid_errors_i[2]),
            ],
            pid_d: [i32::from(telemetry.pid_errors_d[0]), i32::from(telemetry.pid_errors_d[1]), 0],
            pid_s: <[i32; Self::RPY_AXIS_COUNT]>::default(),
            pid_k: <[i32; Self::RPY_AXIS_COUNT]>::default(),
            rc_commands: telemetry.rc_commands,
            setpoints: [telemetry.setpoints[0], telemetry.setpoints[1], telemetry.setpoints[2], 0],
            gyro: (telemetry.gyro_rps.to_degrees()).into(),
            gyro_unfiltered: (telemetry.gyro_rps_unfiltered.to_degrees()).into(),
            acc: (telemetry.acc * 4096.0).into(),
            #[cfg(feature = "magnetometer")]
            mag: <[i16; Self::XYZ_AXIS_COUNT]>::default(),
            #[allow(clippy::cast_possible_truncation)]
            orientation: if telemetry.orientation.w > 0.0 {
                [
                    (telemetry.orientation.x * TO_I16) as i16,
                    (telemetry.orientation.y * TO_I16) as i16,
                    (telemetry.orientation.z * TO_I16) as i16,
                ]
            } else {
                [
                    (-telemetry.orientation.x * TO_I16) as i16,
                    (-telemetry.orientation.y * TO_I16) as i16,
                    (-telemetry.orientation.z * TO_I16) as i16,
                ]
            },
            motor: telemetry.motor_commands,
            #[cfg(feature = "dshot_telemetry")]
            erpm: telemetry.motor_rpm,
            debug: [telemetry.debug[0], telemetry.debug[1], telemetry.debug[2], 0, 0, 0, 0, 0],
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
        is_full::<SlowData>();
        is_full::<GpsData>();
        is_full::<GpsPosition>();
        is_full::<MainData>();
    }
    #[test]
    fn new() {
        let slow_data = SlowData::new();
        assert_eq!(0, slow_data.flight_mode_flags);
        //assert_eq!(152, core::mem::size_of::<MainState>());

        let main_data = MainData::new();
        assert_eq!(0, main_data.time_us);

        let gps_data = GpsData::new();
        assert_eq!(0, gps_data.time_of_week_ms);
    }
}
