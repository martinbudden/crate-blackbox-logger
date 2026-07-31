use crate::field_definitions::{
    FieldCondition as FC, FieldEncoding as FE, FieldPredictor as FP, FieldSign as FS, MainFieldDefinition as MFD,
    SimpleFieldDefinition as SFD,
};

#[cfg(feature = "gps")]
use crate::field_definitions::ConditionalFieldDefinition as CFD;

impl SFD {
    pub const SLOW_FIELD_COUNT: usize = 5;
    #[cfg(feature = "gps")]
    pub const GPS_H_FIELD_COUNT: usize = 4;

    #[allow(unused)]
    pub fn find_by_name(name: &str) -> Option<&'static Self> {
        BLACKBOX_SLOW_FIELDS.iter().find(|field| field.name == name)
    }
}

#[cfg(feature = "gps")]
impl CFD {
    pub const GPS_G_FIELD_COUNT: usize = 11;

    #[allow(unused)]
    pub fn find_by_name(name: &str) -> Option<&'static Self> {
        BLACKBOX_GPS_G_FIELDS.iter().find(|field| field.name == name)
    }
}

impl MFD {
    #[allow(unused)]
    pub fn find_by_name(name: &str) -> Option<&'static Self> {
        BLACKBOX_MAIN_FIELDS.iter().find(|field| field.name == name)
    }
}

#[rustfmt::skip]
pub static BLACKBOX_SLOW_FIELDS: [SFD; SFD::SLOW_FIELD_COUNT] = [
    SFD { name: "flight_mode_flags",          name_index: -1, is_signed: FS::Unsigned, predict: FP::Zero, encode: FE::UnsignedVb },
    SFD { name: "state_flags",                name_index: -1, is_signed: FS::Unsigned, predict: FP::Zero, encode: FE::UnsignedVb },
    SFD { name: "failsafe_phase",             name_index: -1, is_signed: FS::Unsigned, predict: FP::Zero, encode: FE::Tag2_3S32 },
    SFD { name: "rx_signal_received",         name_index: -1, is_signed: FS::Unsigned, predict: FP::Zero, encode: FE::Tag2_3S32 },
    SFD { name: "rx_flight_channel_is_valid", name_index: -1, is_signed: FS::Unsigned, predict: FP::Zero, encode: FE::Tag2_3S32 },
];

// GPS home frame
#[rustfmt::skip]
#[cfg(feature = "gps")]
pub static BLACKBOX_GPS_H_FIELDS: [SFD; SFD::GPS_H_FIELD_COUNT] = [
    SFD { name: "GPS_home",       name_index: 0, is_signed: FS::Signed,   predict: FP::Zero, encode: FE::SignedVb },
    SFD { name: "GPS_home",       name_index: 1, is_signed: FS::Signed,   predict: FP::Zero, encode: FE::SignedVb },
    SFD { name: "GPS_home",       name_index: 2, is_signed: FS::Signed,   predict: FP::Zero, encode: FE::SignedVb },
    SFD { name: "GPS_home_epoch", name_index:-1, is_signed: FS::Unsigned, predict: FP::Zero, encode: FE::UnsignedVb },
];

// GPS position/velocity frame
#[rustfmt::skip]
#[cfg(feature = "gps")]
pub static BLACKBOX_GPS_G_FIELDS: [CFD; CFD::GPS_G_FIELD_COUNT] = [
    CFD { name: "time",              name_index:-1, is_signed: FS::Unsigned, predict: FP::LastMainFrameTime, encode: FE::UnsignedVb, condition: FC::NOT_LOGGING_EVERY_FRAME },
    CFD { name: "GPS_numSat",        name_index:-1, is_signed: FS::Unsigned, predict: FP::Zero,      encode: FE::UnsignedVb, condition: FC::ALWAYS },
    CFD { name: "GPS_coord",         name_index: 0, is_signed: FS::Signed,   predict: FP::HomeCoord, encode: FE::SignedVb,   condition: FC::ALWAYS },
    CFD { name: "GPS_coord",         name_index: 1, is_signed: FS::Signed,   predict: FP::HomeCoord, encode: FE::SignedVb,   condition: FC::ALWAYS },
    CFD { name: "GPS_altitude",      name_index:-1, is_signed: FS::Signed,   predict: FP::Zero,      encode: FE::SignedVb,   condition: FC::ALWAYS },
    CFD { name: "GPS_speed",         name_index:-1, is_signed: FS::Unsigned, predict: FP::Zero,      encode: FE::UnsignedVb, condition: FC::ALWAYS },
    CFD { name: "GPS_ground_course", name_index:-1, is_signed: FS::Unsigned, predict: FP::Zero,      encode: FE::UnsignedVb, condition: FC::ALWAYS },
    CFD { name: "GPS_velned",        name_index: 0, is_signed: FS::Signed,   predict: FP::Zero,      encode: FE::SignedVb,   condition: FC::ALWAYS },
    CFD { name: "GPS_velned",        name_index: 1, is_signed: FS::Signed,   predict: FP::Zero,      encode: FE::SignedVb,   condition: FC::ALWAYS },
    CFD { name: "GPS_velned",        name_index: 2, is_signed: FS::Signed,   predict: FP::Zero,      encode: FE::SignedVb,   condition: FC::ALWAYS },
    CFD { name: "GPS_time",          name_index:-1, is_signed: FS::Unsigned, predict: FP::Zero,      encode: FE::UnsignedVb, condition: FC::ALWAYS },
];

#[rustfmt::skip]
pub static BLACKBOX_MAIN_FIELDS: &[MFD] = &[
    // loopIteration doesn't appear in p_frames since it always increments
    MFD { name: "loopIteration", name_index: -1, is_signed: FS::Unsigned, i_predict: FP::Zero, i_encode: FE::UnsignedVb, p_predict: FP::Inc, p_encode: FE::Null, condition: FC::ALWAYS },
    // Time advances pretty steadily so the p_frame prediction is a straight line
    MFD { name: "time", name_index: -1, is_signed: FS::Unsigned, i_predict: FP::Zero, i_encode: FE::UnsignedVb, p_predict: FP::StraightLine, p_encode: FE::SignedVb, condition: FC::ALWAYS },

    MFD { name: "axisP", name_index: 0, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::SignedVb, condition: FC::PID },
    MFD { name: "axisP", name_index: 1, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::SignedVb, condition: FC::PID },
    MFD { name: "axisP", name_index: 2, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::SignedVb, condition: FC::PID },
    // iterms get special packed encoding in p_frames:
    MFD { name: "axisI", name_index: 0, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::Tag2_3S32, condition: FC::PID },
    MFD { name: "axisI", name_index: 1, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::Tag2_3S32, condition: FC::PID },
    MFD { name: "axisI", name_index: 2, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::Tag2_3S32, condition: FC::PID },
    MFD { name: "axisD", name_index: 0, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::SignedVb, condition: FC::PID_D_ROLL },
    MFD { name: "axisD", name_index: 1, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::SignedVb, condition: FC::PID_D_PITCH },
    MFD { name: "axisD", name_index: 2, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::SignedVb, condition: FC::PID_D_YAW },
    // PID K terms use F (feedforward) suffix, for Betaflight compatibility.
    MFD { name: "axisF", name_index: 0, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::SignedVb, condition: FC::PID_K },
    MFD { name: "axisF", name_index: 1, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::SignedVb, condition: FC::PID_K },
    MFD { name: "axisF", name_index: 2, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::SignedVb, condition: FC::PID_K },
    MFD { name: "axisS", name_index: 0, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::SignedVb, condition: FC::PID_S_ROLL },
    MFD { name: "axisS", name_index: 1, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::SignedVb, condition: FC::PID_S_PITCH },
    MFD { name: "axisS", name_index: 2, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::SignedVb, condition: FC::PID_S_YAW },

    // rc_commands are encoded together as a group in p_frames:
    MFD { name: "rcCommand", name_index: 0, is_signed: FS::Signed,   i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::Tag8_4S16, condition: FC::RC_COMMANDS },
    MFD { name: "rcCommand", name_index: 1, is_signed: FS::Signed,   i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::Tag8_4S16, condition: FC::RC_COMMANDS },
    MFD { name: "rcCommand", name_index: 2, is_signed: FS::Signed,   i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::Tag8_4S16, condition: FC::RC_COMMANDS },
    MFD { name: "rcCommand", name_index: 3, is_signed: FS::Unsigned, i_predict: FP::MinThrottle, i_encode: FE::UnsignedVb, p_predict: FP::Previous, p_encode: FE::Tag8_4S16, condition: FC::RC_COMMANDS },
    // setpoint
    MFD { name: "setpoint", name_index: 0, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::Tag8_4S16, condition: FC::SETPOINT },
    MFD { name: "setpoint", name_index: 1, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::Tag8_4S16, condition: FC::SETPOINT },
    MFD { name: "setpoint", name_index: 2, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::Tag8_4S16, condition: FC::SETPOINT },
    MFD { name: "setpoint", name_index: 3, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::Tag8_4S16, condition: FC::SETPOINT },

    MFD { name: "vbatLatest",     name_index: -1, is_signed: FS::Unsigned, i_predict: FP::VBatRef, i_encode: FE::Neg14bit,   p_predict: FP::Previous, p_encode: FE::Tag8_8SVb, condition: FC::BATTERY_VOLTAGE },
    MFD { name: "amperageLatest", name_index: -1, is_signed: FS::Signed,   i_predict: FP::Zero,    i_encode: FE::SignedVb,   p_predict: FP::Previous, p_encode: FE::Tag8_8SVb, condition: FC::BATTERY_CURRENT },
    MFD { name: "BaroAlt",        name_index: -1, is_signed: FS::Signed,   i_predict: FP::Zero,    i_encode: FE::SignedVb,   p_predict: FP::Previous, p_encode: FE::Tag8_8SVb, condition: FC::BAROMETER },
    MFD { name: "surfaceRaw",     name_index: -1, is_signed: FS::Signed,   i_predict: FP::Zero,    i_encode: FE::SignedVb,   p_predict: FP::Previous, p_encode: FE::Tag8_8SVb, condition: FC::RANGEFINDER },
    MFD { name: "rssi",           name_index: -1, is_signed: FS::Unsigned, i_predict: FP::Zero,    i_encode: FE::UnsignedVb, p_predict: FP::Previous, p_encode: FE::Tag8_8SVb, condition: FC::RSSI },

    MFD { name: "magADC", name_index: 0, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::Tag8_8SVb, condition: FC::MAGNETOMETER },
    MFD { name: "magADC", name_index: 1, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::Tag8_8SVb, condition: FC::MAGNETOMETER },
    MFD { name: "magADC", name_index: 2, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::Tag8_8SVb, condition: FC::MAGNETOMETER },

    MFD { name: "gyroADC", name_index: 0, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::SignedVb, condition: FC::GYRO },
    MFD { name: "gyroADC", name_index: 1, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::SignedVb, condition: FC::GYRO },
    MFD { name: "gyroADC", name_index: 2, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Previous, p_encode: FE::SignedVb, condition: FC::GYRO },

    MFD { name: "gyroUnfilt", name_index: 0, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::GYRO_UNFILTERED },
    MFD { name: "gyroUnfilt", name_index: 1, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::GYRO_UNFILTERED },
    MFD { name: "gyroUnfilt", name_index: 2, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::GYRO_UNFILTERED },

    MFD { name: "accSmooth", name_index: 0, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::ACC },
    MFD { name: "accSmooth", name_index: 1, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::ACC },
    MFD { name: "accSmooth", name_index: 2, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::ACC },

    MFD { name: "imuQuaternion", name_index: 0, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::ATTITUDE },
    MFD { name: "imuQuaternion", name_index: 1, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::ATTITUDE },
    MFD { name: "imuQuaternion", name_index: 2, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::ATTITUDE },

    MFD { name: "debug", name_index: 0, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::DEBUG },
    MFD { name: "debug", name_index: 1, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::DEBUG },
    MFD { name: "debug", name_index: 2, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::DEBUG },
    MFD { name: "debug", name_index: 3, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::DEBUG },
    MFD { name: "debug", name_index: 4, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::DEBUG },
    MFD { name: "debug", name_index: 5, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::DEBUG },
    MFD { name: "debug", name_index: 6, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::DEBUG },
    MFD { name: "debug", name_index: 7, is_signed: FS::Signed, i_predict: FP::Zero, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::DEBUG },

    MFD { name: "motor", name_index: 0, is_signed: FS::Signed, i_predict: FP::MinMotor, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::AT_LEAST_MOTORS_1 },
    MFD { name: "motor", name_index: 1, is_signed: FS::Unsigned, i_predict: FP::Motor0, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::AT_LEAST_MOTORS_2 },
    MFD { name: "motor", name_index: 2, is_signed: FS::Unsigned, i_predict: FP::Motor0, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::AT_LEAST_MOTORS_3 },
    MFD { name: "motor", name_index: 3, is_signed: FS::Unsigned, i_predict: FP::Motor0, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::AT_LEAST_MOTORS_4 },
    #[cfg(feature = "eight_motors")]
    MFD { name: "motor", name_index: 4, is_signed: FS::Unsigned, i_predict: FP::Motor0, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::AT_LEAST_MOTORS_5 },
    #[cfg(feature = "eight_motors")]
    MFD { name: "motor", name_index: 5, is_signed: FS::Unsigned, i_predict: FP::Motor0, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::AT_LEAST_MOTORS_6 },
    #[cfg(feature = "eight_motors")]
    MFD { name: "motor", name_index: 6, is_signed: FS::Unsigned, i_predict: FP::Motor0, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::AT_LEAST_MOTORS_7 },
    #[cfg(feature = "eight_motors")]
    MFD { name: "motor", name_index: 7, is_signed: FS::Unsigned, i_predict: FP::Motor0, i_encode: FE::SignedVb, p_predict: FP::Average2, p_encode: FE::SignedVb, condition: FC::AT_LEAST_MOTORS_8 },

    #[cfg(feature = "dshot_telemetry")]
    MFD { name: "eRPM", name_index: 0, is_signed: FS::Unsigned, i_predict: FP::Zero, i_encode: FE::UnsignedVb, p_predict: FP::Previous, p_encode: FE::SignedVb, condition: FC::MOTOR_1_HAS_RPM },
    #[cfg(feature = "dshot_telemetry")]
    MFD { name: "eRPM", name_index: 1, is_signed: FS::Unsigned, i_predict: FP::Zero, i_encode: FE::UnsignedVb, p_predict: FP::Previous, p_encode: FE::SignedVb, condition: FC::MOTOR_2_HAS_RPM },
    #[cfg(feature = "dshot_telemetry")]
    MFD { name: "eRPM", name_index: 2, is_signed: FS::Unsigned, i_predict: FP::Zero, i_encode: FE::UnsignedVb, p_predict: FP::Previous, p_encode: FE::SignedVb, condition: FC::MOTOR_3_HAS_RPM },
    #[cfg(feature = "dshot_telemetry")]
    MFD { name: "eRPM", name_index: 3, is_signed: FS::Unsigned, i_predict: FP::Zero, i_encode: FE::UnsignedVb, p_predict: FP::Previous, p_encode: FE::SignedVb, condition: FC::MOTOR_4_HAS_RPM },
    #[cfg(all(feature = "dshot_telemetry", feature = "eight_motors"))]
    MFD { name: "eRPM", name_index: 4, is_signed: FS::Unsigned, i_predict: FP::Zero, i_encode: FE::UnsignedVb, p_predict: FP::Previous, p_encode: FE::SignedVb, condition: FC::MOTOR_5_HAS_RPM },
    #[cfg(all(feature = "dshot_telemetry", feature = "eight_motors"))]
    MFD { name: "eRPM", name_index: 5, is_signed: FS::Unsigned, i_predict: FP::Zero, i_encode: FE::UnsignedVb, p_predict: FP::Previous, p_encode: FE::SignedVb, condition: FC::MOTOR_6_HAS_RPM },
    #[cfg(all(feature = "dshot_telemetry", feature = "eight_motors"))]
    MFD { name: "eRPM", name_index: 6, is_signed: FS::Unsigned, i_predict: FP::Zero, i_encode: FE::UnsignedVb, p_predict: FP::Previous, p_encode: FE::SignedVb, condition: FC::MOTOR_7_HAS_RPM },
    #[cfg(all(feature = "dshot_telemetry", feature = "eight_motors"))]
    MFD { name: "eRPM", name_index: 7, is_signed: FS::Unsigned, i_predict: FP::Zero, i_encode: FE::UnsignedVb, p_predict: FP::Previous, p_encode: FE::SignedVb, condition: FC::MOTOR_8_HAS_RPM },
];
