use crate::data::GpsPosition;
use vqm::{Quaternionf32, Vector3df32, Vector4df32};
/*
Current Estimates for GyroPidMessage:
AccData (4x f32 ): 16 bytes
GyroData (4x f32 ): 16 bytes
GyroUnfiltered (4x f32 ): 16 bytes
Orientation 16 bytes
MotorCommands (4x u16): 8 bytes
Setpoints (3x f32): 12 bytes
PID Errors (3x f32): 12 bytes
 */

/// Blackbox telemetry data, updated at approximately 1kHz.
/// Limit this to 128 bytes.
/// On a 32-bit ARM processor, a memcpy of 128 bytes takes roughly 32 to 64 clock cycles.
/// At 200MHz, that is less than 0.4 microseconds.
/// Even with the overhead of the Watch mutex (critical section), this is well under 1μs of total time to dispatch that data.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct GyroPidMessage {
    // Vector3df32 is padded to 16 bytes
    pub acc: Vector3df32,
    pub gyro_rps: Vector3df32,
    pub gyro_rps_unfiltered: Vector3df32,
    pub orientation: Quaternionf32,
    pub motor_commands: Vector4df32,
    pub pid_errors_p: [f32; Self::RPY_AXIS_COUNT],
    pub pid_errors_i: [f32; Self::RPY_AXIS_COUNT],
    pub pid_errors_d: [f32; Self::RP_AXIS_COUNT],
    pub time_us: u32,
    pub debug: [i16; Self::DEBUG_COUNT], // only 6 debug fields, to keep structure size down to 128 bytes
}

//#[cfg(not(any(feature = "servos", feature = "eight_motors")))]
const _: () = assert!(core::mem::size_of::<GyroPidMessage>() == 128);

impl GyroPidMessage {
    pub const RPY_AXIS_COUNT: usize = 3;
    pub const RP_AXIS_COUNT: usize = 2;
    // we use 6 debug fields to keep size of BlackboxTelemetry down to 128 bytes, there are 2 more debug fields in SetpointMessage
    pub const DEBUG_COUNT: usize = 6;
}

impl GyroPidMessage {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            acc: Vector3df32::new(0.0, 0.0, 0.0),
            gyro_rps: Vector3df32::new(0.0, 0.0, 0.0),
            gyro_rps_unfiltered: Vector3df32::new(0.0, 0.0, 0.0),
            orientation: Quaternionf32::new(0.0, 0.0, 0.0, 0.0),
            motor_commands: Vector4df32::new(0.0, 0.0, 0.0, 0.0),
            pid_errors_p: [0f32; Self::RPY_AXIS_COUNT],
            pid_errors_i: [0f32; Self::RPY_AXIS_COUNT],
            pid_errors_d: [0f32; Self::RP_AXIS_COUNT],
            time_us: 0,
            debug: [0i16; Self::DEBUG_COUNT],
        }
    }
}

impl Default for GyroPidMessage {
    fn default() -> Self {
        Self::new()
    }
}

/// Message to send setpoint data between tasks.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct SetpointMessage {
    pub setpoints: [f32; Self::SETPOINT_COUNT],
    pub pid_errors_s: [f32; Self::RPY_AXIS_COUNT],
    pub pid_errors_k: [f32; Self::RPY_AXIS_COUNT],
    pub rc_commands: [i16; Self::RC_COMMAND_COUNT],
    #[cfg(feature = "dshot_telemetry")]
    pub motor_rpm_d2: [i16; Self::MAX_SUPPORTED_MOTOR_COUNT], // motor rpm divided by 2
    #[cfg(feature = "servos")]
    pub servos: [i16; Self::MAX_SUPPORTED_SERVO_COUNT],
    pub debug: [i16; Self::SETPOINT_DEBUG_COUNT],

    pub time_us: u32,
    pub flight_mode_flags: u32,
    pub state_flags: u8,
    pub failsafe_phase: u8,
    pub rx_signal_received: bool,
    pub rx_flight_channel_is_valid: bool,
}

impl SetpointMessage {
    pub const RC_COMMAND_COUNT: usize = 4;
    pub const SETPOINT_COUNT: usize = 4;
    pub const THROTTLE: usize = 3;
    pub const SETPOINT_DEBUG_COUNT: usize = 8 - GyroPidMessage::DEBUG_COUNT; // the remaining debug fields
    pub const RPY_AXIS_COUNT: usize = 3;

    #[cfg(feature = "eight_motors")]
    pub const MAX_SUPPORTED_MOTOR_COUNT: usize = 8;
    #[cfg(not(feature = "eight_motors"))]
    pub const MAX_SUPPORTED_MOTOR_COUNT: usize = 4;
    #[cfg(feature = "servos")]
    pub const MAX_SUPPORTED_SERVO_COUNT: usize = 8; // ailerons, elevator, rudder, throttle (which may be controlled by a servo, if the motor is an internal combustion engine)
}

impl SetpointMessage {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            setpoints: [0.0; Self::SETPOINT_COUNT],
            pid_errors_s: [0.0; Self::RPY_AXIS_COUNT],
            pid_errors_k: [0.0; Self::RPY_AXIS_COUNT],
            rc_commands: [0i16; Self::RC_COMMAND_COUNT],
            #[cfg(feature = "dshot_telemetry")]
            motor_rpm_d2: [0i16; Self::MAX_SUPPORTED_MOTOR_COUNT],
            #[cfg(feature = "servos")]
            servos: [0i16; Self::MAX_SUPPORTED_SERVO_COUNT],
            debug: [0i16; Self::SETPOINT_DEBUG_COUNT],
            time_us: 0,
            flight_mode_flags: 0,
            state_flags: 0,
            failsafe_phase: 0,
            rx_signal_received: false,
            rx_flight_channel_is_valid: false,
        }
    }
}

impl Default for SetpointMessage {
    fn default() -> Self {
        Self::new()
    }
}

/// Message to send GPS data between tasks.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct GpsMessage {
    /// GPS time of week in ms.
    pub time_of_week_ms: u32,
    /// interval between GPS solutions in ms.
    pub interval_ms: u32,
    /// current position.
    pub position: GpsPosition,
    /// north velocity, cm/s.
    pub velocity_north_cmps: i16,
    /// east velocity, cm/s.
    pub velocity_east_cmps: i16,
    /// down velocity, cm/s.
    pub velocity_down_cmps: i16,
    /// speed in cm/s.
    pub speed3d_cmps: i16,
    /// speed in cm/s.
    pub ground_speed_cmps: i16,
    /// Heading 2D in 10ths of a degree.
    pub ground_course_deci_degrees: i16,
    pub satellite_count: u8,
}

impl GpsMessage {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            time_of_week_ms: 0,
            interval_ms: 0,
            position: GpsPosition::new(),
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

impl Default for GpsMessage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<GyroPidMessage>();
        is_full::<SetpointMessage>();
        is_full::<GpsMessage>();
    }
    #[test]
    fn sizeof() {
        assert_eq!(128, core::mem::size_of::<GyroPidMessage>());
        #[cfg(all(feature = "dshot_telemetry", not(any(feature = "servos", feature = "eight_motors"))))]
        assert_eq!(72, core::mem::size_of::<SetpointMessage>());
        assert_eq!(36, core::mem::size_of::<GpsMessage>());
    }
    #[test]
    fn gyro_pid_message_new() {
        let telemetry_data = GyroPidMessage::default();
        assert_eq!(0, telemetry_data.time_us);
    }
}
