use vqm::{Quaternionf32, Vector3df32};
/*
Current Estimates for FastTelemetryData:
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
pub struct BlackboxTelemetry {
    // Vector3df32 is padded to 16 bytes
    pub acc: Vector3df32,
    pub gyro_rps: Vector3df32,
    pub gyro_rps_unfiltered: Vector3df32,
    pub orientation: Quaternionf32,
    pub motor_commands: [i16; Self::MAX_SUPPORTED_MOTOR_COUNT],
    #[cfg(feature = "dshot_telemetry")]
    pub motor_rpm: [i16; Self::MAX_SUPPORTED_MOTOR_COUNT],
    #[cfg(feature = "servos")]
    pub servos: [i16; Self::MAX_SUPPORTED_SERVO_COUNT],
    pub rc_commands: [i16; Self::RC_COMMAND_COUNT],
    pub setpoints: [i16; Self::SETPOINT_COUNT],
    pub debug: [i16; Self::DEBUG_COUNT], // only 3 debug fields, to keep structure size down to 128 bytes
    pub pid_errors_p: [i16; Self::RPY_AXIS_COUNT],
    pub pid_errors_i: [i16; Self::RPY_AXIS_COUNT],
    pub pid_errors_k: [i16; Self::RPY_AXIS_COUNT],
    #[cfg(any(feature = "servos", feature = "eight_motors"))]
    pub pid_errors_s: [i16; Self::RPY_AXIS_COUNT], // we've breached 128 bytes, so we now have 144 bytes available
    pub pid_errors_d: [i16; Self::RP_AXIS_COUNT],
    pub time_us: u32,
}

#[cfg(not(any(feature = "servos", feature = "eight_motors")))]
const _: () = assert!(std::mem::size_of::<BlackboxTelemetry>() == 128);

impl BlackboxTelemetry {
    pub const RPY_AXIS_COUNT: usize = 3;
    pub const RP_AXIS_COUNT: usize = 2;
    #[cfg(feature = "eight_motors")]
    pub const MAX_SUPPORTED_MOTOR_COUNT: usize = 8;
    #[cfg(not(feature = "eight_motors"))]
    pub const MAX_SUPPORTED_MOTOR_COUNT: usize = 4;
    #[cfg(feature = "servos")]
    pub const MAX_SUPPORTED_SERVO_COUNT: usize = 8; // ailerons, elevator, rudder, throttle (which may be controlled by a servo, if the motor is an internal combustion engine)
    pub const RC_COMMAND_COUNT: usize = 4;
    pub const DEBUG_COUNT: usize = 3; // we use 3 debug fields to keep size of BlackboxTelemetry down to 128 bytes, there are 5 more debug fields in BlackboxSlowTelemetry
    pub const SETPOINT_COUNT: usize = 4;
    pub const THROTTLE: usize = 3;
}

impl BlackboxTelemetry {
    pub fn new() -> Self {
        Self {
            acc: Vector3df32::default(),
            gyro_rps: Vector3df32::default(),
            gyro_rps_unfiltered: Vector3df32::default(),
            orientation: Quaternionf32::default(),
            motor_commands: <[i16; Self::MAX_SUPPORTED_MOTOR_COUNT]>::default(),
            #[cfg(feature = "dshot_telemetry")]
            motor_rpm: <[i16; Self::MAX_SUPPORTED_MOTOR_COUNT]>::default(),
            rc_commands: <[i16; Self::RC_COMMAND_COUNT]>::default(),
            #[cfg(feature = "servos")]
            servos: <[i16; Self::MAX_SUPPORTED_SERVO_COUNT]>::default(),
            setpoints: <[i16; Self::SETPOINT_COUNT]>::default(),
            debug: <[i16; Self::DEBUG_COUNT]>::default(),
            pid_errors_p: <[i16; Self::RPY_AXIS_COUNT]>::default(),
            pid_errors_i: <[i16; Self::RPY_AXIS_COUNT]>::default(),
            pid_errors_k: <[i16; Self::RPY_AXIS_COUNT]>::default(),
            #[cfg(any(feature = "servos", feature = "eight_motors"))]
            pid_errors_s: <[i16; Self::RPY_AXIS_COUNT]>::default(),
            pid_errors_d: <[i16; Self::RP_AXIS_COUNT]>::default(),
            time_us: 0,
        }
    }
}

impl Default for BlackboxTelemetry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct BlackboxSlowTelemetry {
    pub flight_mode_flags: u32,
    pub state_flags: u8,
    pub failsafe_phase: u8,
    pub rx_signal_received: bool,
    pub rx_flight_channel_is_valid: bool,
    pub debug: [i16; Self::SLOW_DEBUG_COUNT],
}

impl BlackboxSlowTelemetry {
    pub const SLOW_DEBUG_COUNT: usize = 5;
}

impl BlackboxSlowTelemetry {
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

impl Default for BlackboxSlowTelemetry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct BlackboxGpsTelemetry {
    pub flags: u32,
}

impl BlackboxGpsTelemetry {
    pub const COUNT: usize = 1;
}

impl BlackboxGpsTelemetry {
    pub fn new() -> Self {
        Self { flags: 0 }
    }
}

impl Default for BlackboxGpsTelemetry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(unused)]
    fn is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    /*fn is_config<
        T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq + Serialize + for<'a> Deserialize<'a>,
    >() {
    }*/

    #[test]
    fn normal_types() {
        is_full::<BlackboxTelemetry>();
        is_full::<BlackboxSlowTelemetry>();
    }
    #[test]
    fn fast_new() {
        #[cfg(not(any(feature = "servos", feature = "eight_motors")))]
        assert_eq!(128, core::mem::size_of::<BlackboxTelemetry>());
        let telemetry_data = BlackboxTelemetry::default();
        assert_eq!(0, telemetry_data.time_us);
    }
}
