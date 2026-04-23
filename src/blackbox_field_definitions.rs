pub trait FieldHeader {
    fn name(&self) -> &'static str;
    fn field_name_index(&self) -> i8;
    fn is_signed(&self) -> u8;
    fn predict(&self) -> u8;
    fn encode(&self) -> u8;
}

// Simple fields, used for S-Frames and H-Frames
#[repr(C)]
pub struct SimpleFieldDefinition {
    pub name: &'static str,
    pub field_name_index: i8,
    pub is_signed: u8,
    pub predict: u8,
    pub encode: u8,
}

impl FieldHeader for SimpleFieldDefinition {
    fn name(&self) -> &'static str {
        self.name
    }
    fn field_name_index(&self) -> i8 {
        self.field_name_index
    }
    fn is_signed(&self) -> u8 {
        self.is_signed
    }
    fn predict(&self) -> u8 {
        self.predict
    }
    fn encode(&self) -> u8 {
        self.encode
    }
}

// Conditional fields, used for G-Frames
#[repr(C)]
pub struct ConditionalFieldDefinition {
    pub name: &'static str,
    pub field_name_index: i8,
    pub is_signed: u8,
    pub predict: u8,
    pub encode: u8,
    pub condition: u8,
}

impl FieldHeader for ConditionalFieldDefinition {
    fn name(&self) -> &'static str {
        self.name
    }
    fn field_name_index(&self) -> i8 {
        self.field_name_index
    }
    fn is_signed(&self) -> u8 {
        self.is_signed
    }
    fn predict(&self) -> u8 {
        self.predict
    }
    fn encode(&self) -> u8 {
        self.encode
    }
}

#[repr(C)]
pub struct MainFieldDefinition {
    pub name: &'static str,
    pub field_name_index: i8,
    pub is_signed: u8,
    // i_frame settings
    pub i_predict: u8,
    pub i_encode: u8,
    // p_frame settings
    pub p_predict: u8,
    pub p_encode: u8,
    pub condition: u8,
}

impl FieldHeader for MainFieldDefinition {
    fn name(&self) -> &'static str {
        self.name
    }
    fn field_name_index(&self) -> i8 {
        self.field_name_index
    }
    fn is_signed(&self) -> u8 {
        self.is_signed
    }
    fn predict(&self) -> u8 {
        self.i_predict
    }
    fn encode(&self) -> u8 {
        self.i_encode
    }
}

pub struct FieldSign;
impl FieldSign {
    pub const UNSIGNED: u8 = 0;
    pub const SIGNED: u8 = 1;
}

pub struct FieldPredictor;
impl FieldPredictor {
    pub const ZERO: u8 = 0;
    pub const PREVIOUS: u8 = 1;
    pub const STRAIGHT_LINE: u8 = 2;
    pub const AVERAGE_2: u8 = 3;
    pub const MIN_THROTTLE: u8 = 4;
    pub const MOTOR_0: u8 = 5;
    pub const INC: u8 = 6;
    #[cfg(feature = "gps")]
    pub const HOME_COORD: u8 = 7;
    #[cfg(feature = "servos")]
    pub const S_1500: u8 = 8;
    pub const VBATREF: u8 = 9;
    pub const LAST_MAIN_FRAME_TIME: u8 = 10;
    pub const MIN_MOTOR: u8 = 11;
}

pub struct FieldEncoding;
impl FieldEncoding {
    pub const SIGNED_VB: u8 = 0;
    pub const UNSIGNED_VB: u8 = 1;
    pub const NEG_14BIT: u8 = 3;
    pub const TAG8_8SVB: u8 = 6;
    pub const TAG2_3S32: u8 = 7;
    pub const TAG8_4S16: u8 = 8;
    pub const ZERO: u8 = 9;
    #[allow(unused)]
    pub const TAG2_3SVARIABLE: u8 = 10;
}

pub struct FieldCondition;
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
    #[cfg(feature = "servos")]
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
    pub const PID_S_ROLL: u8 = 30;
    pub const PID_S_PITCH: u8 = 31;
    pub const PID_S_YAW: u8 = 32;

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

pub struct LogFieldSelect;
impl LogFieldSelect {
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
    pub const MAGNETOMETER: u32 = 0x10000;
    pub const MOTOR: u32 = 0x20000;
    pub const MOTOR_RPM: u32 = 0x40000;
    pub const SERVO: u32 = 0x80000;
    pub const BATTERY_VOLTAGE: u32 = 0x10_0000;
    pub const BATTERY_CURRENT: u32 = 0x20_0000;
    pub const BAROMETER: u32 = 0x40_0000;
    pub const RANGEFINDER: u32 = 0x80_0000;
    pub const GPS: u32 = 0x100_0000;
}
