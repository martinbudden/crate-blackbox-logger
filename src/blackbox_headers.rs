use crate::{BLACKBOX_MAIN_FIELDS, BLACKBOX_SLOW_FIELDS};
use vqm::BitSet64;

use crate::{ConditionalFieldDefinition, FieldHeader, MainFieldDefinition, SimpleFieldDefinition};
use crate::BlackboxWriter;

// Helper to handle the "H Field X type: val,val" formatting
// Notice the closure now takes (&mut dyn BlackboxWriter, &T)
fn write_field_line<'a, T, I, F>(writer: &mut dyn BlackboxWriter, frame_type: char, label: &str, fields: I, mut op: F)
where
    T: 'a,
    I: Iterator<Item = &'a T>,
    F: FnMut(&mut dyn BlackboxWriter, &T),
{
    writer.write_str("H Field ");
    writer.write_char(frame_type);
    writer.write_char(' ');
    writer.write_str(label);
    writer.write_char(':');

    for (i, field) in fields.enumerate() {
        if i > 0 {
            writer.write_char(',');
        }
        op(writer, field);
    }
    writer.write_char('\n');
}

fn write_common_field_lines<'a, T, P>(writer: &mut dyn BlackboxWriter, frame_type: char, fields: &[T], mut predicate: P)
where
    T: FieldHeader + 'a,
    P: FnMut(&T) -> bool, // The condition closure
{
    // Helper to create a fresh iterator for each line
    // We use .iter().filter() to apply the condition

    // Name line
    write_field_line(writer, frame_type, "name", fields.iter().filter(|&f| predicate(f)), |w, f| {
        w.write_str(f.name());
        let index = f.field_name_index();
        if index >= 0 {
            w.write_char('[');
            w.write_u8_ascii(index.cast_unsigned());
            w.write_char(']');
        }
    });

    // Signed line
    write_field_line(writer, frame_type, "signed", fields.iter().filter(|&f| predicate(f)), |w, f| {
        w.write_u8_ascii(f.is_signed());
    });

    // Predictor line
    write_field_line(writer, frame_type, "predictor", fields.iter().filter(|&f| predicate(f)), |w, f| {
        w.write_u8_ascii(f.predict());
    });

    // Encoder line
    write_field_line(writer, frame_type, "encoding", fields.iter().filter(|&f| predicate(f)), |w, f| {
        w.write_u8_ascii(f.encode());
    });
}

/// Simple header, used for S and H frames.
// H Field S name:flight_mode_flags,state_flags,failsafe_phase,rx_signal_received,rx_flight_channel_is_valid
// H Field S signed:   0,0,0,0,0
// H Field S predictor:0,0,0,0,0
// H Field S encoding: 1,1,7,7,7
// or
// H Field H name:GPS_home[0],GPS_home[1]
// ..
pub fn write_simple_header(writer: &mut dyn BlackboxWriter, frame_type: char, fields: &[SimpleFieldDefinition]) {
    // Only include fields that are "active"
    write_common_field_lines(writer, frame_type, fields, |_f| true);
}

/// Conditional header, used for G frames.
// H Field G name:time,GPS_numSat,GPS_coord[0],GPS_coord[1],GPS_altitude,GPS_speed,GPS_ground_course,GPS_velned[0],GPS_velned[1],GPS_velned[2]
// H Field G signed:0,0,1,1,1,0,0,1,1,1
// H Field G predictor:10,0,7,7,0,0,0,0,0,0
// H Field G encoding:1,1,0,0,0,1,1,0,0,0
pub fn write_conditional_header(
    writer: &mut dyn BlackboxWriter,
    frame_type: char,
    fields: &[ConditionalFieldDefinition],
    conditions: BitSet64,
) {
    let filter = |f: &ConditionalFieldDefinition| conditions.test(f.condition);
    write_common_field_lines(writer, frame_type, fields, &filter);

    /*let filtered = fields.iter().filter(|&f| filter(f));
    write_header_line(writer, frame_type, "condition", filtered, |w, f| {
        w.write_u8_ascii(f.condition);
    });*/
}

/// Main header, used for i_frames and p_frames.
//0: H Field I name:loopIteration,time,axisP[0],axisP[1],axisP[2],axisI[0],axisI[1],axisI[2],axisD[0],axisD[1],axisD[2],rc_command[0],rc_command[1],rc_command[2],rc_command[3],vbat_latest,amperage_latest,gyro_adc[0],gyro_adc[1],gyro_adc[2],motor[0],motor[1],motor[2],motor[3]
//1: H Field I signed:   0,0,1,1,1,1,1,1,1,1,1,1,1,1,0,0,0,1,1,1, 0,0,0,0
//2: H Field I predictor:0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,9,0,0,0,0,11,5,5,5
//3: H Field I encoding: 1,1,0,0,0,0,0,0,0,0,0,0,0,0,1,3,1,0,0,0, 1,0,0,0
//4: H Field P predictor:6,2,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,3,3,3, 3,3,3,3
//5: H Field P encoding: 9,0,0,0,0,7,7,7,0,0,0,8,8,8,8,6,6,0,0,0, 0,0,0,0
pub fn write_main_header(writer: &mut dyn BlackboxWriter, fields: &[MainFieldDefinition], conditions: BitSet64) {
    let filter = |f: &MainFieldDefinition| conditions.test(f.condition);
    write_common_field_lines(writer, 'I', fields, &filter);

    let filtered = fields.iter().filter(|&f| filter(f));
    write_field_line(writer, 'P', "predictor", filtered, |w, f| {
        w.write_u8_ascii(f.p_predict);
    });

    let filtered = fields.iter().filter(|&f| filter(f));
    write_field_line(writer, 'P', "encoding", filtered, |w, f| {
        w.write_u8_ascii(f.p_encode);
    });

    /*let filtered = fields.iter().filter(|&f| filter(f));
    write_header_line(writer, 'P', "condition", filtered, |w, f| {
        w.write_u8_ascii(f.condition);
    });*/
}

pub fn write_header(writer: &mut dyn BlackboxWriter, conditions: BitSet64) {
    writer.write_str("H Product:Blackbox flight data recorder by Nicholas Sherlock\n");
    writer.write_str("H Data version:2\n");

    write_main_header(writer, BLACKBOX_MAIN_FIELDS, conditions);
    write_simple_header(writer, 'S', &BLACKBOX_SLOW_FIELDS);
    writer.write_str("H Firmware type:Cleanflight\n");
    writer.write_str("H Firmware revision:Betaflight 3.3.1 (611bc70f8) REVOLT\n");
    writer.write_str("H Firmware date:Mar 21 2018 03:14:05\n");
    writer.write_str("H Log start datetime:0000-01-01T00:00:00.000+00:00\n");
    writer.write_str("H Craft name:Mark1\n");
    writer.write_str("H I interval:256\n");
    writer.write_str("H P interval:1/8\n");
    writer.write_str("H P denom:32\n");
    writer.write_str("H minthrottle:1070\n");
    writer.write_str("H maxthrottle:2000\n");
    writer.write_str("H gyro_scale:0x3f800000\n");
    writer.write_str("H motorOutput:158,2047\n");
    writer.write_str("H acc_1G:4096\n");
    writer.write_str("H vbat_scale:110\n");
    writer.write_str("H vbatcellvoltage:33,35,43\n");
    writer.write_str("H vbatref:113\n");
    writer.write_str("H currentSensor:0,235\n");
    writer.write_str("H looptime:125\n");
    writer.write_str("H gyro_sync_denom:1\n");
    writer.write_str("H pid_process_denom:1\n");
    writer.write_str("H debug_mode:0\n");
    writer.write_str("H features:541130760\n");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BLACKBOX_MAIN_FIELDS, BLACKBOX_SLOW_FIELDS, FieldCondition};

    // A simple mock writer that captures output into a byte slice
    struct MockWriter<'a> {
        buffer: &'a mut [u8],
        pos: usize,
    }

    impl BlackboxWriter for MockWriter<'_> {
    fn write_byte(&mut self, byte: u8) {
        if self.pos < self.buffer.len() {
            self.buffer[self.pos] = byte;
            self.pos += 1;
        }
    }
    }

    #[test]
    fn slow_fields_header() {
        let mut buffer = [0u8; 1024];
        let mut writer = MockWriter { buffer: &mut buffer, pos: 0 };

        // Generate headers for the SLOW_FIELDS array defined earlier
        write_simple_header(&mut writer, 'S', &BLACKBOX_SLOW_FIELDS);

        // Convert the written portion to a string for validation
        #[allow(clippy::unwrap_used)]
        let result = core::str::from_utf8(&writer.buffer[..writer.pos]).unwrap();

        // Expected output segments:
        // Names: flight_mode_flags,state_flags,failsafe_phase,rx_signal_received,rx_flight_channel_is_valid
        // Predictors: 0,0,0,0,0 (All are PREDICT(ZERO) = 0)
        // Encodings: 1,1,7,7,7 (UNSIGNED_VB=1, TAG2_3S32=7)

        // Print for manual inspection (if running with `cargo test -- --nocapture`)
        //println!("{result}");
        assert!(result.contains(
            "H Field S name:flight_mode_flags,state_flags,failsafe_phase,rx_signal_received,rx_flight_channel_is_valid"
        ));
        assert!(result.contains("H Field S predictor:0,0,0,0,0"));
        assert!(result.contains("H Field S encoding:1,1,7,7,7"));
    }
    #[test]
    fn main_fields_header() {
        let mut buffer = [0u8; 2048];
        let mut writer = MockWriter { buffer: &mut buffer, pos: 0 };

        // Generate headers for the SLOW_FIELDS array defined earlier
        let mut conditions = BitSet64::new();
        _ = conditions.set(FieldCondition::ALWAYS);
        _ = conditions.set(FieldCondition::AT_LEAST_MOTORS_1);
        _ = conditions.set(FieldCondition::AT_LEAST_MOTORS_2);
        _ = conditions.set(FieldCondition::AT_LEAST_MOTORS_3);
        _ = conditions.set(FieldCondition::AT_LEAST_MOTORS_4);
        _ = conditions.set(FieldCondition::PID);
        _ = conditions.set(FieldCondition::PID_K);
        _ = conditions.set(FieldCondition::PID_D_PITCH);
        _ = conditions.set(FieldCondition::PID_D_ROLL);
        _ = conditions.set(FieldCondition::GYRO);
        _ = conditions.set(FieldCondition::RC_COMMANDS);
        _ = conditions.set(FieldCondition::BATTERY_CURRENT);
        _ = conditions.set(FieldCondition::BATTERY_VOLTAGE);
        write_main_header(&mut writer, BLACKBOX_MAIN_FIELDS, conditions);

        // Convert the written portion to a string for validation
        #[allow(clippy::unwrap_used)]
        let result = core::str::from_utf8(&writer.buffer[..writer.pos]).unwrap();
        assert!(result.contains("H Field I name:loopIteration,time"));

        // Expected output segments:
        // Names: flight_mode_flags,state_flags,failsafe_phase,rx_signal_received,rx_flight_channel_is_valid
        // Predictors: 0,0,0,0,0 (All are PREDICT(ZERO) = 0)
        // Encodings: 1,1,7,7,7 (UNSIGNED_VB=1, TAG2_3S32=7)

        // Print for manual inspection (if running with `cargo test -- --nocapture`)
        //println!("{result}");
    }
    #[test]
    fn main_header() {
        use crate::blackbox::{Blackbox, BlackboxConfig};
        let mut buffer = [0u8; 2048];
        let mut writer = MockWriter { buffer: &mut buffer, pos: 0 };

        // Generate headers for the SLOW_FIELDS array defined earlier
        /*let mut conditions = BitSet64::new();
        _ = conditions.set(FieldCondition::ALWAYS);
        _ = conditions.set(FieldCondition::AT_LEAST_MOTORS_1);
        _ = conditions.set(FieldCondition::AT_LEAST_MOTORS_2);
        _ = conditions.set(FieldCondition::AT_LEAST_MOTORS_3);
        _ = conditions.set(FieldCondition::AT_LEAST_MOTORS_4);
        _ = conditions.set(FieldCondition::PID);
        _ = conditions.set(FieldCondition::PID_K);
        _ = conditions.set(FieldCondition::PID_D_PITCH);
        _ = conditions.set(FieldCondition::PID_D_ROLL);
        _ = conditions.set(FieldCondition::GYRO);
        _ = conditions.set(FieldCondition::RC_COMMANDS);
        _ = conditions.set(FieldCondition::BATTERY_CURRENT);
        _ = conditions.set(FieldCondition::BATTERY_VOLTAGE);
        write_header(&mut writer, conditions);*/
        let mut blackbox = Blackbox::new();
        let config = BlackboxConfig::new();
        blackbox.init(config);
        write_header(&mut writer, blackbox.conditions);

        // Convert the written portion to a string for validation
        #[allow(clippy::unwrap_used)]
        let result = core::str::from_utf8(&writer.buffer[..writer.pos]).unwrap();
        assert!(result.contains("H Field I name:loopIteration,time"));

        // Expected output segments:
        // Names: flight_mode_flags,state_flags,failsafe_phase,rx_signal_received,rx_flight_channel_is_valid
        // Predictors: 0,0,0,0,0 (All are PREDICT(ZERO) = 0)
        // Encodings: 1,1,7,7,7 (UNSIGNED_VB=1, TAG2_3S32=7)

        // Print for manual inspection (if running with `cargo test -- --nocapture`)
        println!("{result}");
    }
}
/*
Explanation of the test:
MockWriter: Simulates a serial port or file stream by writing bytes into a stack-allocated array.
core::str::from_utf8: Safely converts the buffer back to a string so we can use standard string assertions.
Validation:
It checks that the Names are comma-separated and match the BLACKBOX_SLOW_FIELDS definition.
It verifies the Predictors are all 0 (mapping to FlightLogFieldPredictor::ZERO).
It verifies the Encodings match the numeric constants we defined (e.g., TAG2_3S32 is 7).
To run this, you would use:
cargo test */
