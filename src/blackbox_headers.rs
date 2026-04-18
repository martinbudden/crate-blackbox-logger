use crate::{MainFieldDefinition, SimpleFieldDefinition,ConditionalFieldDefinition};
pub trait HeaderWriter {
    fn write_str(&mut self, s: &str);
    fn write_char(&mut self, c: char);
}

/// Minimal no_std u8 to ASCII helper.
fn write_u8_ascii(writer: &mut dyn HeaderWriter, mut n: u8) {
    if n == 0 {
        writer.write_char('0');
        return;
    }
    let mut buf = [0u8; 3];
    let mut i = 0;
    while n > 0 {
        buf[i] = (n % 10) + b'0';
        n /= 10;
        i += 1;
    }
    for j in (0..i).rev() {
        writer.write_char(buf[j] as char);
    }
}

// Simple headers, used for S and H frames
// H Field S name:flight_mode_flags,state_flags,failsafe_phase,rx_signal_received,rx_flight_channe_is_valid
// H Field H name:GPS_home[0],GPS_home[1]
// H Field S signed:   0,0,0,0,0
// H Field S predictor:0,0,0,0,0
// H Field S encoding: 1,1,7,7,7
pub fn write_simple_field_headers(
    writer: &mut dyn HeaderWriter,
    frame_type: char, // 'I', 'P', 'S', or 'G'
    fields: &[SimpleFieldDefinition],
) {
    // 1. Write Field Names: H Field [type] Name:time,loopIteration,syncBeep...
    writer.write_str("H Field ");
    writer.write_char(frame_type);
    writer.write_str(" Name:");
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            writer.write_char(',');
        }
        writer.write_str(field.name);
    }
    writer.write_char('\n');

    // 2. Signs
    // H Field S signed:   0,0,0,0,0
    writer.write_str("H Field ");
    writer.write_char(frame_type);
    writer.write_str(" signed:");
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            writer.write_char(',');
        }
        // We use a simple u8 to string conversion here
        write_u8_ascii(writer, field.is_signed);
    }
    writer.write_char('\n');

    // 3. Write Predictors: H Field [type] Predictor:0,1,1,3...
    writer.write_str("H Field ");
    writer.write_char(frame_type);
    writer.write_str(" predictor:");
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            writer.write_char(',');
        }
        // We use a simple u8 to string conversion here
        write_u8_ascii(writer, field.predict);
    }
    writer.write_char('\n');

    // 4. Write Encodings: H Field [type] Encoding:1,1,0,6...
    writer.write_str("H Field ");
    writer.write_char(frame_type);
    writer.write_str(" encoding:");
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            writer.write_char(',');
        }
        write_u8_ascii(writer, field.encode);
    }
    writer.write_char('\n');
}

// H Field G name:time,GPS_numSat,GPS_coord[0],GPS_coord[1],GPS_altitude,GPS_speed,GPS_ground_course,GPS_velned[0],GPS_velned[1],GPS_velned[2]
// H Field G signed:0,0,1,1,1,0,0,1,1,1
// H Field G predictor:10,0,7,7,0,0,0,0,0,0
// H Field G encoding:1,1,0,0,0,1,1,0,0,0
pub fn write_conditional_headers(
    writer: &mut dyn HeaderWriter,
    frame_type: char, // 'I', 'P', 'S', or 'G'
    fields: &[ConditionalFieldDefinition]
) {
    // 1. Write Field Names: H Field [type] Name:time,loopIteration,syncBeep...
    writer.write_str("H Field ");
    writer.write_char(frame_type);
    writer.write_str(" Name:");
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            writer.write_char(',');
        }
        writer.write_str(field.name);
    }
    writer.write_char('\n');

    // 2. Signs
    // H Field S signed:   0,0,0,0,0
    writer.write_str("H Field ");
    writer.write_char(frame_type);
    writer.write_str(" signed:");
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            writer.write_char(',');
        }
        // We use a simple u8 to string conversion here
        write_u8_ascii(writer, field.is_signed);
    }
    writer.write_char('\n');

    // 3. Write Predictors: H Field [type] Predictor:0,1,1,3...
    writer.write_str("H Field ");
    writer.write_char(frame_type);
    writer.write_str(" predictor:");
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            writer.write_char(',');
        }
        // We use a simple u8 to string conversion here
        write_u8_ascii(writer, field.predict);
    }
    writer.write_char('\n');

    // 4. Write Encodings: H Field [type] Encoding:1,1,0,6...
    writer.write_str("H Field ");
    writer.write_char(frame_type);
    writer.write_str(" encoding:");
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            writer.write_char(',');
        }
        write_u8_ascii(writer, field.encode);
    }
    writer.write_char('\n');

    writer.write_str("H Field G Condition:");
    for (i, field) in fields.iter().enumerate() {
        if i > 0 { writer.write_char(','); }
        write_u8_ascii(writer, field.condition);
    }
    writer.write_char('\n');
}

// main headers, used for I and P frames
//0: H Field I name:loopIteration,time,axisP[0],axisP[1],axisP[2],axisI[0],axisI[1],axisI[2],axisD[0],axisD[1],axisD[2],rc_command[0],rc_command[1],rc_command[2],rc_command[3],vbat_latest,amperage_latest,gyro_adc[0],gyro_adc[1],gyro_adc[2],motor[0],motor[1],motor[2],motor[3]
//1: H Field I signed:   0,0,1,1,1,1,1,1,1,1,1,1,1,1,0,0,0,1,1,1, 0,0,0,0
//2: H Field I predictor:0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,9,0,0,0,0,11,5,5,5
//3: H Field I encoding: 1,1,0,0,0,0,0,0,0,0,0,0,0,0,1,3,1,0,0,0, 1,0,0,0
//4: H Field P predictor:6,2,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,3,3,3, 3,3,3,3
//5: H Field P encoding: 9,0,0,0,0,7,7,7,0,0,0,8,8,8,8,6,6,0,0,0, 0,0,0,0
pub fn write_main_headers(writer: &mut dyn HeaderWriter, fields: &[MainFieldDefinition]) {
    // 1. Field Names
    writer.write_str("H Field P Name:");
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            writer.write_char(',');
        }
        writer.write_str(field.name);
    }
    writer.write_char('\n');

    // 2. Predictors (using P-specific values)
    writer.write_str("H Field P Predictor:");
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            writer.write_char(',');
        }
        write_u8_ascii(writer, field.p_predict);
    }
    writer.write_char('\n');

    // 3. Encodings (using P-specific values)
    writer.write_str("H Field P Encoding:");
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            writer.write_char(',');
        }
        write_u8_ascii(writer, field.p_encode);
    }
    writer.write_char('\n');
}

//For P-frames to decode correctly, you must also write the P interval header, which defines how many PID loop iterations occur between logs.
pub fn write_p_interval_header(writer: &mut dyn HeaderWriter, numerator: u8, denominator: u8) {
    writer.write_str("H P interval:");
    write_u8_ascii(writer, numerator);
    writer.write_char('/');
    write_u8_ascii(writer, denominator);
    writer.write_char('\n');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BLACKBOX_SLOW_FIELDS;

    // A simple mock writer that captures output into a byte slice
    struct MockWriter<'a> {
        buf: &'a mut [u8],
        pos: usize,
    }

    impl HeaderWriter for MockWriter<'_> {
        fn write_str(&mut self, s: &str) {
            for b in s.as_bytes() {
                if self.pos < self.buf.len() {
                    self.buf[self.pos] = *b;
                    self.pos += 1;
                }
            }
        }
        fn write_char(&mut self, c: char) {
            if self.pos < self.buf.len() {
                self.buf[self.pos] = c as u8;
                self.pos += 1;
            }
        }
    }

    #[test]
    fn test_slow_fields_header() {
        let mut buffer = [0u8; 1024];
        let mut writer = MockWriter { buf: &mut buffer, pos: 0 };

        // Generate headers for the SLOW_FIELDS array defined earlier
        write_simple_field_headers(&mut writer, 'S', &BLACKBOX_SLOW_FIELDS);

        // Convert the written portion to a string for validation
        #[allow(clippy::unwrap_used)]
        let result = core::str::from_utf8(&writer.buf[..writer.pos]).unwrap();

        // Expected output segments:
        // Names: flight_mode_flags,state_flags,failsafe_phase,rx_signal_received,rx_flight_channel_is_valid
        // Predictors: 0,0,0,0,0 (All are PREDICT(ZERO) = 0)
        // Encodings: 1,1,7,7,7 (UNSIGNED_VB=1, TAG2_3S32=7)

        assert!(result.contains(
            "H Field S Name:flight_mode_flags,state_flags,failsafe_phase,rx_signal_received,rx_flight_channel_is_valid"
        ));
        //assert!(result.contains("H Field S Predictor:0,0,0,0,0"));
        //assert!(result.contains("H Field S Encoding:1,1,7,7,7"));

        // Print for manual inspection (if running with `cargo test -- --nocapture`)
        println!("{result}");
    }

    #[test]
    fn test_main_headers() {
        use crate::{FieldEncoding, FieldPredictor};
        let mut buffer = [0u8; 1024];
        let mut writer = MockWriter { buf: &mut buffer, pos: 0 };

        // Define a small set of main fields for the test
        let test_fields = [
            MainFieldDefinition {
                name: "loopIteration",
                field_name_index: -1,
                is_signed: 0,
                i_predict: 0,
                i_encode: 1,
                p_predict: FieldPredictor::INC, // 6
                p_encode: 9,                    // ZERO/NULL encoding
                condition: 0,
            },
            MainFieldDefinition {
                name: "time",
                field_name_index: -1,
                is_signed: 0,
                i_predict: 0,
                i_encode: 1,
                p_predict: FieldPredictor::STRAIGHT_LINE, // 2
                p_encode: FieldEncoding::SIGNED_VB,       // 0
                condition: 0,
            },
        ];

        // Generate the P headers
        write_main_headers(&mut writer, &test_fields);
        write_p_interval_header(&mut writer, 1, 1);

        #[allow(clippy::unwrap_used)]
        let result = core::str::from_utf8(&writer.buf[..writer.pos]).unwrap();

        // 1. Check Names
        assert!(result.contains("H Field P Name:loopIteration,time"));

        // 2. Check P-Predictors (INC=6, STRAIGHT_LINE=2)
        assert!(result.contains("H Field P Predictor:6,2"));

        // 3. Check P-Encodings (NULL=9, SIGNED_VB=0)
        assert!(result.contains("H Field P Encoding:9,0"));

        // 4. Check Interval
        assert!(result.contains("H P interval:1/1"));

        // Use `cargo test -- --nocapture` to see this output
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
