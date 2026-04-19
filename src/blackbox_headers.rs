use crate::{ConditionalFieldDefinition, FieldHeader, MainFieldDefinition, SimpleFieldDefinition};
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

fn write_common_header_lines<T: FieldHeader>(writer: &mut dyn HeaderWriter, frame_type: char, fields: &[T]) {
    // Name line
    write_header_line(writer, frame_type, "name", fields, |w, f| {
        w.write_str(f.name());
        let index = f.field_name_index();
        if index >= 0 {
            w.write_char('[');
            write_u8_ascii(w, index.cast_unsigned());
            w.write_char(']');
        }
    });

    // Signed line
    write_header_line(writer, frame_type, "signed", fields, |w, f| {
        write_u8_ascii(w, f.is_signed());
    });

    // Predictor line
    write_header_line(writer, frame_type, "predictor", fields, |w, f| {
        write_u8_ascii(w, f.predict());
    });
    // Encoder line
    write_header_line(writer, frame_type, "encode", fields, |w, f| {
        write_u8_ascii(w, f.encode());
    });
}

// Helper to handle the "H Field X type: val,val" formatting
// Notice the closure now takes (&mut dyn HeaderWriter, &T)
fn write_header_line<T, F>(writer: &mut dyn HeaderWriter, frame_type: char, label: &str, fields: &[T], mut op: F)
where
    F: FnMut(&mut dyn HeaderWriter, &T), // <--- Added writer here
{
    writer.write_str("H Field ");
    writer.write_char(frame_type);
    writer.write_char(' ');
    writer.write_str(label);
    writer.write_char(':');
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            writer.write_char(',');
        }
        op(writer, field); // <--- Pass it in here
    }
    writer.write_char('\n');
}

// Simple headers, used for S and H frames
// H Field S name:flight_mode_flags,state_flags,failsafe_phase,rx_signal_received,rx_flight_channel_is_valid
// H Field H name:GPS_home[0],GPS_home[1]
// H Field S signed:   0,0,0,0,0
// H Field S predictor:0,0,0,0,0
// H Field S encoding: 1,1,7,7,7
pub fn write_simple_header(writer: &mut dyn HeaderWriter, frame_type: char, fields: &[SimpleFieldDefinition]) {
    write_common_header_lines(writer, frame_type, fields);
}

// H Field G name:time,GPS_numSat,GPS_coord[0],GPS_coord[1],GPS_altitude,GPS_speed,GPS_ground_course,GPS_velned[0],GPS_velned[1],GPS_velned[2]
// H Field G signed:0,0,1,1,1,0,0,1,1,1
// H Field G predictor:10,0,7,7,0,0,0,0,0,0
// H Field G encoding:1,1,0,0,0,1,1,0,0,0
pub fn write_conditional_header(
    writer: &mut dyn HeaderWriter,
    frame_type: char,
    fields: &[ConditionalFieldDefinition],
) {
    write_common_header_lines(writer, frame_type, fields);

    // Condition line
    write_header_line(writer, frame_type, "condition", fields, |w, f| {
        write_u8_ascii(w, f.condition);
    });
}

// main headers, used for i_frames and p_frames
//0: H Field I name:loopIteration,time,axisP[0],axisP[1],axisP[2],axisI[0],axisI[1],axisI[2],axisD[0],axisD[1],axisD[2],rc_command[0],rc_command[1],rc_command[2],rc_command[3],vbat_latest,amperage_latest,gyro_adc[0],gyro_adc[1],gyro_adc[2],motor[0],motor[1],motor[2],motor[3]
//1: H Field I signed:   0,0,1,1,1,1,1,1,1,1,1,1,1,1,0,0,0,1,1,1, 0,0,0,0
//2: H Field I predictor:0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,9,0,0,0,0,11,5,5,5
//3: H Field I encoding: 1,1,0,0,0,0,0,0,0,0,0,0,0,0,1,3,1,0,0,0, 1,0,0,0
//4: H Field P predictor:6,2,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,3,3,3, 3,3,3,3
//5: H Field P encoding: 9,0,0,0,0,7,7,7,0,0,0,8,8,8,8,6,6,0,0,0, 0,0,0,0
pub fn write_main_header(writer: &mut dyn HeaderWriter, fields: &[MainFieldDefinition]) {
    write_common_header_lines(writer, 'I', fields);

    write_header_line(writer, 'P', "predictor", fields, |w, f| {
        write_u8_ascii(w, f.p_predict);
    });
    write_header_line(writer, 'P', "encoding", fields, |w, f| {
        write_u8_ascii(w, f.p_encode);
    });
    write_header_line(writer, 'P', "condition", fields, |w, f| {
        write_u8_ascii(w, f.condition);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BLACKBOX_MAIN_FIELDS, BLACKBOX_SLOW_FIELDS};

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
    fn slow_fields_header() {
        let mut buffer = [0u8; 1024];
        let mut writer = MockWriter { buf: &mut buffer, pos: 0 };

        // Generate headers for the SLOW_FIELDS array defined earlier
        write_simple_header(&mut writer, 'S', &BLACKBOX_SLOW_FIELDS);

        // Convert the written portion to a string for validation
        #[allow(clippy::unwrap_used)]
        let result = core::str::from_utf8(&writer.buf[..writer.pos]).unwrap();

        // Expected output segments:
        // Names: flight_mode_flags,state_flags,failsafe_phase,rx_signal_received,rx_flight_channel_is_valid
        // Predictors: 0,0,0,0,0 (All are PREDICT(ZERO) = 0)
        // Encodings: 1,1,7,7,7 (UNSIGNED_VB=1, TAG2_3S32=7)

        // Print for manual inspection (if running with `cargo test -- --nocapture`)
        println!("{result}");
        assert!(result.contains(
            "H Field S name:flight_mode_flags,state_flags,failsafe_phase,rx_signal_received,rx_flight_channel_is_valid"
        ));
        assert!(result.contains("H Field S predictor:0,0,0,0,0"));
        assert!(result.contains("H Field S encode:1,1,7,7,7"));
    }
    #[test]
    fn main_fields_header() {
        let mut buffer = [0u8; 2048];
        let mut writer = MockWriter { buf: &mut buffer, pos: 0 };

        // Generate headers for the SLOW_FIELDS array defined earlier
        write_main_header(&mut writer, BLACKBOX_MAIN_FIELDS);

        // Convert the written portion to a string for validation
        #[allow(clippy::unwrap_used)]
        let result = core::str::from_utf8(&writer.buf[..writer.pos]).unwrap();

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
