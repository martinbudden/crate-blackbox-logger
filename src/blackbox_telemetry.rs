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
    // Vector3df32 and Quaternionf32 are actually padded to 16 bytes, so the unfiltered gyro is split so the whole structure fits in 64 bytes.
    pub acc: Vector3df32,
    pub gyro_rps: Vector3df32,
    pub gyro_rps_unfiltered: Vector3df32,
    pub orientation: Quaternionf32,
    pub time_us: u32,
    pub rc_commands: [i16; 4],
    pub motor_commands: [i16; 4],
    pub motor_rpm: [i16; 4],
    pub debug: [i16; 8],
    pub setpoints: [i16; 3],
    pub pid_errors: [i16; 3],
}

impl BlackboxTelemetry {
    pub fn new() -> Self {
        Self {
            acc: Vector3df32::default(),
            gyro_rps: Vector3df32::default(),
            gyro_rps_unfiltered: Vector3df32::default(),
            orientation: Quaternionf32::default(),
            time_us: 0,
            rc_commands: <[i16; 4]>::default(),
            motor_commands: <[i16; 4]>::default(),
            motor_rpm: <[i16; 4]>::default(),
            debug: <[i16; 8]>::default(),
            setpoints: <[i16; 3]>::default(),
            pid_errors: <[i16; 3]>::default(),
        }
    }
}

impl Default for BlackboxTelemetry {
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
    }
    #[test]
    fn fast_new() {
        assert_eq!(128, core::mem::size_of::<BlackboxTelemetry>());
        let telemetry_data = BlackboxTelemetry::default();
        assert_eq!(0, telemetry_data.time_us);
    }
}
