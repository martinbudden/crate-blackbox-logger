// Simple fields, used for S-Frames and H-Frames
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct SimpleFieldDefinition {
    pub name: &'static str,
    pub name_index: i8,
    pub is_signed: FieldSign,
    pub predict: FieldPredictor,
    pub encode: FieldEncoding,
}

// Conditional fields, used for G-Frames
#[cfg(feature = "gps")]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct ConditionalFieldDefinition {
    pub name: &'static str,
    pub name_index: i8,
    pub is_signed: FieldSign,
    pub predict: FieldPredictor,
    pub encode: FieldEncoding,
    pub condition: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct MainFieldDefinition {
    pub name: &'static str,
    pub name_index: i8,
    pub is_signed: FieldSign,
    // i_frame settings
    pub i_predict: FieldPredictor,
    pub i_encode: FieldEncoding,
    // p_frame settings
    pub p_predict: FieldPredictor,
    pub p_encode: FieldEncoding,
    pub condition: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(u8)]
pub enum FieldSign {
    #[default]
    Unsigned = 0,
    Signed = 1,
}

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(u8)]
pub enum FieldPredictor {
    #[default]
    Zero = 0,
    Previous = 1,
    StraightLine = 2,
    Average2 = 3,
    MinThrottle = 4,
    Motor0 = 5,
    Inc = 6,
    HomeCoord = 7,
    S1500 = 8,
    VBatRef = 9,
    LastMainFrameTime = 10,
    MinMotor = 11,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(u8)]
pub enum FieldEncoding {
    #[default]
    SignedVb = 0,
    UnsignedVb = 1,
    Neg14bit = 3,
    Tag8_8SVb = 6,
    Tag2_3S32 = 7,
    Tag8_4S16 = 8,
    Null = 9,
    #[allow(unused)]
    Tag2_3SVariable = 10,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FieldCondition;

#[allow(unused)]
impl FieldCondition {
    pub const ALWAYS: u8 = 0;

    pub const AT_LEAST_MOTORS_1: u8 = 1;
    pub const AT_LEAST_MOTORS_2: u8 = 2;
    pub const AT_LEAST_MOTORS_3: u8 = 3;
    pub const AT_LEAST_MOTORS_4: u8 = 4;
    pub const AT_LEAST_MOTORS_5: u8 = 5;
    pub const AT_LEAST_MOTORS_6: u8 = 6;
    pub const AT_LEAST_MOTORS_7: u8 = 7;
    pub const AT_LEAST_MOTORS_8: u8 = 8;

    pub const MOTOR_1_HAS_RPM: u8 = 9;
    pub const MOTOR_2_HAS_RPM: u8 = 10;
    pub const MOTOR_3_HAS_RPM: u8 = 11;
    pub const MOTOR_4_HAS_RPM: u8 = 12;
    pub const MOTOR_5_HAS_RPM: u8 = 13;
    pub const MOTOR_6_HAS_RPM: u8 = 14;
    pub const MOTOR_7_HAS_RPM: u8 = 15;
    pub const MOTOR_8_HAS_RPM: u8 = 16;

    pub const SERVOS: u8 = 17;
    pub const TRICOPTER: u8 = 18;

    pub const MAGNETOMETER: u8 = 19;
    pub const BAROMETER: u8 = 20;
    pub const BATTERY_VOLTAGE: u8 = 21;
    pub const BATTERY_CURRENT: u8 = 22;
    pub const RANGEFINDER: u8 = 23;
    pub const RSSI: u8 = 24;

    pub const PID: u8 = 25;
    pub const PID_K: u8 = 26;
    pub const PID_D_ROLL: u8 = 27;
    pub const PID_D_PITCH: u8 = 28;
    pub const PID_D_YAW: u8 = 29;
    pub const PID_S_ROLL: u8 = 30; // equivalent to Betaflight NONZERO_WING_S_0
    pub const PID_S_PITCH: u8 = 31; // equivalent to Betaflight NONZERO_WING_S_1
    pub const PID_S_YAW: u8 = 32; // equivalent to Betaflight NONZERO_WING_S_2

    pub const RC_COMMANDS: u8 = 33;
    pub const SETPOINT: u8 = 34;

    pub const NOT_LOGGING_EVERY_FRAME: u8 = 35;

    pub const GYRO: u8 = 36;
    pub const GYRO_UNFILTERED: u8 = 37;
    pub const ACC: u8 = 38;
    pub const ATTITUDE: u8 = 39;
    pub const DEBUG: u8 = 40;

    pub const NEVER: u8 = 41;

    pub const FIRST: u8 = Self::ALWAYS;
    pub const LAST: u8 = Self::NEVER;
}

/// Empty struct to define field selection flags.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FieldSelect;

#[allow(missing_docs)]
impl FieldSelect {
    pub const DEBUG: u32 = 0x0001;
    pub const PID: u32 = 0x0002;
    pub const PID_DTERM_ROLL: u32 = 0x0004;
    pub const PID_DTERM_PITCH: u32 = 0x0008;
    pub const PID_DTERM_YAW: u32 = 0x0010;
    pub const PID_STERM_ROLL: u32 = 0x0020;
    pub const PID_STERM_PITCH: u32 = 0x0040;
    pub const PID_STERM_YAW: u32 = 0x0080;
    pub const PID_KTERM: u32 = 0x0100;
    pub const SETPOINT: u32 = 0x0200;
    pub const RC_COMMANDS: u32 = 0x0400;
    pub const RSSI: u32 = 0x0800;
    pub const GYRO: u32 = 0x1000;
    pub const GYRO_UNFILTERED: u32 = 0x2000;
    pub const ACCELEROMETER: u32 = 0x4000;
    pub const ATTITUDE: u32 = 0x8000;
    pub const MAGNETOMETER: u32 = 0x1_0000;
    pub const MOTOR: u32 = 0x2_0000;
    pub const MOTOR_RPM: u32 = 0x4_0000;
    pub const SERVO: u32 = 0x8_0000;
    pub const BATTERY_VOLTAGE: u32 = 0x10_0000;
    pub const BATTERY_CURRENT: u32 = 0x20_0000;
    pub const BAROMETER: u32 = 0x40_0000;
    pub const RANGEFINDER: u32 = 0x80_0000;
    pub const GPS: u32 = 0x100_0000;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<SimpleFieldDefinition>();
        #[cfg(feature = "gps")]
        is_full::<ConditionalFieldDefinition>();
        is_full::<MainFieldDefinition>();
        is_full::<FieldSign>();
        is_full::<FieldEncoding>();
        is_full::<FieldCondition>();
        is_full::<FieldSelect>();
    }
}
