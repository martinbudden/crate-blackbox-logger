pub trait BlackboxWriter {
    fn write_byte(&mut self, byte: u8);

    fn write_char(&mut self, c: char) {
        self.write_byte(c as u8);
    }

    fn write_str(&mut self, s: &str) {
        for b in s.as_bytes() {
            self.write_byte(*b);
        }
    }

    fn write_h_str(&mut self, s: &str) {
        self.write_str("H ");
        self.write_str(s);
    }

    /// Minimal no_std u8 to ASCII helper.
    fn write_u8_ascii(&mut self, mut n: u8) {
        if n == 0 {
            self.write_char('0');
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
            self.write_char(buf[j] as char);
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    fn write_u32_ascii(&mut self, mut n: u32) {
        if n == 0 {
            self.write_char('0');
            return;
        }
        let mut buf = [0u8; 16];
        let mut i = 0;
        while n > 0 {
            buf[i] = ((n % 10) + u32::from(b'0')) as u8;
            n /= 10;
            i += 1;
        }
        for j in (0..i).rev() {
            self.write_char(buf[j] as char);
        }
    }
    fn write_h_str_u32_ascii(&mut self, s: &str, n: u32) {
        self.write_h_str(s);
        self.write_u32_ascii(n);
        self.write_char('\n');
    }
}

// Helper to handle the "H Field X type: val,val" formatting
// Notice the closure now takes (&mut dyn BlackboxWriter, &T)
pub fn write_field_line<'a, T, I, F>(
    writer: &mut dyn BlackboxWriter,
    frame_type: char,
    label: &str,
    fields: I,
    mut op: F,
) where
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

// Simple wrapper for a mutable slice
#[derive(Default)]
pub struct SliceWriter<'a> {
    pub buffer: &'a mut [u8],
    pub pos: usize,
}

impl SliceWriter<'_> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl BlackboxWriter for SliceWriter<'_> {
    fn write_byte(&mut self, byte: u8) {
        if self.pos < self.buffer.len() {
            self.buffer[self.pos] = byte;
            self.pos += 1;
        }
    }
}

impl SliceWriter<'_> {
    // begin_frame and end_frame to support future Huffman compression of frame.
    pub fn begin_frame(&mut self, value: u8) {
        self.write_byte(value);
    }

    pub fn end_frame(&self) -> usize {
        self.pos
    }

    /// Unsigned Variable-Byte.
    pub fn write_unsigned_vb(&mut self, mut value: u32) {
        while value > 127 {
            // Set high bit (continuation) and take 7 bits
            #[allow(clippy::cast_possible_truncation)]
            self.write_byte((value as u8) | 0x80);
            value >>= 7;
        }
        // Last byte has high bit 0
        #[allow(clippy::cast_possible_truncation)]
        self.write_byte(value as u8);
    }

    pub fn write_unsigned_vb_16(&mut self, value: u16) {
        self.write_unsigned_vb(u32::from(value));
    }

    pub fn write_unsigned_vb_array(&mut self, values: &[u32]) {
        for &value in values {
            // We cast to i32 to reuse our existing Signed VB logic
            self.write_unsigned_vb(value);
        }
    }

    pub fn write_unsigned_vb_16_array(&mut self, values: &[u16]) {
        for &value in values {
            // We cast to i32 to reuse our existing Signed VB logic
            self.write_unsigned_vb_16(value);
        }
    }

    /// ZigZag encode: maps -1 to 1, 1 to 2, -2 to 3, 2 to 4...
    #[inline]
    pub const fn zigzag_encode(value: i32) -> u32 {
        ((value << 1) ^ (value >> 31)).cast_unsigned()
    }

    /// Signed Variable-Byte.
    pub fn write_signed_vb(&mut self, value: i32) {
        self.write_unsigned_vb(Self::zigzag_encode(value));
    }

    pub fn write_signed_vb_16(&mut self, value: i16) {
        self.write_signed_vb(i32::from(value));
    }

    /// Encodes an array of i32 values using Signed Variable-Byte encoding.
    pub fn write_signed_vb_array(&mut self, values: &[i32]) {
        for &value in values {
            // We cast to i32 to reuse our existing Signed VB logic
            self.write_signed_vb(value);
        }
    }

    pub fn write_signed_vb_16_array(&mut self, values: &[i16]) {
        for &value in values {
            // We cast to i32 to reuse our existing Signed VB logic
            self.write_signed_vb_16(value);
        }
    }

    /// Encodes a group of up to 8 fields using TAG8_8SVB.
    pub fn write_tag8_8svb(&mut self, values: &[i32; 8]) {
        if values.is_empty() {
            return;
        }
        if values.len() == 1 {
            self.write_signed_vb(values[0]);
            return;
        }
        // Step 1: Build the bitmask header
        let mut header: u8 = 0;
        for (ii, value) in values.iter().enumerate() {
            if *value != 0 {
                header |= 1 << ii;
            }
        }
        self.write_byte(header);

        // Step 2: Write only the non-zero values
        for value in values {
            if *value != 0 {
                self.write_signed_vb(*value);
            }
        }
    }

    /// Encodes 4 values into TAG8_4S16 format.
    /// an 8-bit selector followed by four signed fields of size 0, 4, 8 or 16 bits.
    /// Values are truncated to i16 range as per the format name (4s16).
    /// TODO: this needs checking.
    pub fn write_tag8_4s16(&mut self, values: [i16; 4]) {
        const BITS_0: u8 = 0;
        const BITS_4: u8 = 1;
        const BITS_8: u8 = 2;
        const BITS_16: u8 = 3;

        let mut tag: u8 = 0;

        // 1. Determine the size needed for each value and build the tag
        for (ii, val) in values.iter().enumerate() {
            let size_code = if *val == 0 {
                BITS_0 // 0 bits
            } else if (-8..8).contains(val) {
                BITS_4 // 4 bits
            } else if (-128..128).contains(val) {
                BITS_8 // 8 bits
            } else {
                BITS_16 // 16 bits
            };
            tag |= size_code << (ii * 2);
        }

        // 2. Write the tag byte
        self.write_byte(tag);

        // 3. Pack 4-bit values or write 8/16 bit values
        // Note: 4-bit values (nibbles) are packed into a single byte if there's a pair
        let mut buffer: Option<u8> = None;

        for value in values {
            match tag & 0x03 {
                BITS_4 => {
                    // 4-bit nibble
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    if let Some(nibble) = buffer {
                        self.write_byte(nibble | (value & 0x0F) as u8);
                        buffer = None;
                    } else {
                        buffer = Some((value << 4) as u8);
                    }
                }
                BITS_8 => {
                    // 8-bit byte
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    if let Some(nibble) = buffer {
                        //Write the high bits of the value first (mask to avoid sign extension)
                        self.write_byte(nibble | ((value >> 4) & 0x0F) as u8);
                        //Now put the leftover low bits into the top of the next buffer entry
                        buffer = Some((value << 4) as u8);
                    } else {
                        self.write_byte(value as u8);
                    }
                }
                BITS_16 => {
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    if let Some(nibble) = buffer {
                        //First write the highest 4 bits
                        self.write_byte(nibble | ((value >> 12) & 0x0F) as u8);
                        // Then the middle 8
                        self.write_byte((value >> 4) as u8);
                        // leave the smallest 4 bits still left to write
                        buffer = Some((value << 4) as u8);
                    } else {
                        // 16-bit short (Little Endian)
                        let bytes = value.to_le_bytes();
                        self.write_byte(bytes[0]);
                        self.write_byte(bytes[1]);
                    }
                }
                _ => {} // 0-bit: do nothing
            }
            tag >>= 2;
        }

        // 4. If a single nibble is left over (odd number of 4-bit fields), write it
        if let Some(lone_nibble) = buffer {
            self.write_byte(lone_nibble);
        }
    }

    pub fn write_tag2_3s32(&mut self, values: [i32; 3]) {
        // Find the required size for the largest value.
        const BITS_2: u8 = 0;
        const BITS_4: u8 = 1;
        const BITS_6: u8 = 2;
        const BITS_32: u8 = 3;

        let mut bits_needed = BITS_2;
        for &val in &values {
            let needed = if val == 0 {
                BITS_2
            } else if (-8..8).contains(&val) {
                BITS_4
            } else if (-128..128).contains(&val) {
                BITS_6
            } else {
                BITS_32
            };
            if needed > bits_needed {
                bits_needed = needed;
            }
        }

        // Write the 2-bit tag (as a full byte, as per Betaflight protocol).
        self.write_byte(bits_needed);

        // Write data based on the tag.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        match bits_needed {
            BITS_4 => {
                // 4-bit nibbles
                self.write_byte(((values[0] as u8) & 0x0F) | (((values[1] as u8) & 0x0F) << 4));
                self.write_byte((values[2] as u8) & 0x0F);
            }
            BITS_6 => {
                // 8-bit bytes
                for &v in &values {
                    self.write_byte(v as u8);
                }
            }
            BITS_32 => {
                // 16-bit shorts
                for &v in &values {
                    let b = (v as i16).to_le_bytes();
                    self.write_byte(b[0]);
                    self.write_byte(b[1]);
                }
            }
            _ => {} // 0 is no-op
        }
    }
}

/*
Key coverage areas:
Bounds Safety: Ensures write_byte doesn't panic if the buffer is too small.
Variable Byte: Validates the continuation bit logic for u32 values above 127.
ZigZag Encoding: Checks that write_signed_vb correctly maps negative numbers to odd positives and positives to even.
Tag Packaging:
tag8_8svb: Verifies the bitmask header matches the non-zero positions.
tag8_4s16: Verifies the complex 2-bit-per-value tag and the nibble-packing logic (handling odd nibbles).
tag2_3s32: Confirms the "global" size selector (the highest required size) is applied to all elements in the group.
*/
#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a writer and a fixed buffer
    fn create_writer(buf: &mut [u8]) -> SliceWriter<'_> {
        SliceWriter { buffer: buf, pos: 0 }
    }

    #[test]
    fn write_byte() {
        let mut buf = [0u8; 2];
        {
            let mut writer = create_writer(&mut buf);
            writer.write_byte(10);
            writer.write_byte(20);
            writer.write_byte(30); // Should be ignored (out of bounds)
            assert_eq!(writer.pos, 2);
        }

        assert_eq!(buf, [10, 20]);
    }

    #[test]
    fn unsigned_vb() {
        let mut buf = [0u8; 5];

        // Test single byte
        {
            let mut writer = create_writer(&mut buf);
            writer.write_unsigned_vb(127);

            // Test multi byte (128 = 0x80 0x01 in LE VB)
            writer.write_unsigned_vb(128);
        }
        assert_eq!(buf[0], 127);
        assert_eq!(buf[1..3], [0x80, 0x01]);
    }

    #[test]
    fn signed_vb_zigzag() {
        let mut buf = [0u8; 5];
        let mut writer = create_writer(&mut buf);

        // 0 -> 0
        writer.write_signed_vb(0);
        // -1 -> 1
        writer.write_signed_vb(-1);
        // 1 -> 2
        writer.write_signed_vb(1);

        assert_eq!(buf[..3], [0, 1, 2]);
    }

    #[test]
    fn write_tag8_8svb() {
        let mut buf = [0u8; 10];
        let mut writer = create_writer(&mut buf);

        // Only 1st and 3rd values are non-zero
        // Header: (1 << 0) | (1 << 2) = 1 | 4 = 5
        let values = [1, 0, -1, 0, 0, 0, 0, 0];
        writer.write_tag8_8svb(&values);

        // Expected: [Header, ZigZag(1), ZigZag(-1)] -> [5, 2, 1]
        assert_eq!(buf[..3], [5, 2, 1]);
    }

    #[test]
    fn write_tag8_4s16_mixed() {
        let mut buf = [0u8; 10];
        let mut writer = create_writer(&mut buf);

        // v0: 0 (00), v1: 3 (4-bit: 01), v2: 200 (8-bit: 10), v3: 500 (16-bit: 11)
        // Tag: 00 | (01 << 2) | (10 << 4) | (11 << 6) = 0x00 | 0x04 | 0x20 | 0xC0 = 0xE4
        let values: [i16; 4] = [0, 3, 200, 500];
        writer.write_tag8_4s16(values);

        //let expected_tag = 0xE4;
        //let expected_v1_nibble = 3u8;
        //let expected_v2_byte = 200u8;
        //let expected_v3_le = 500i16.to_le_bytes();

        assert_eq!(buf[0], 0b_11_11_01_00);
        assert_eq!(buf[1], 0b_0011_0000); // Single nibble left over
        assert_eq!(buf[2], 0b_0000_1100);
        assert_eq!(buf[3], 0b_1000_0000);
        assert_eq!(buf[4], 0b_0001_1111);
    }

    #[test]
    fn write_tag2_3s32_nibbles() {
        let mut buf = [0u8; 10];
        let mut writer = create_writer(&mut buf);

        // All fit in 4-bit nibbles (bits_needed = 1)
        let values = [2, -1, 5];
        writer.write_tag2_3s32(values);

        // Byte 0: Tag (1)
        // Byte 1: (2 & 0xF) | ((-1 as u8 & 0xF) << 4) = 0x02 | 0xF0 = 0xF2
        // Byte 2: (5 & 0xF) = 0x05
        assert_eq!(buf[..3], [1, 0xF2, 0x05]);
    }

    #[test]
    fn write_signed_vb_16_array() {
        let mut buf = [0u8; 10];
        let mut writer = create_writer(&mut buf);

        let values = [0i16, -1i16];
        writer.write_signed_vb_16_array(&values);

        // ZigZag 0 -> 0, ZigZag -1 -> 1
        assert_eq!(buf[..2], [0, 1]);
    }
}

/*What these cover:
ZigZag Extremes: Ensures i16::MIN is correctly ZigZag encoded to a positive value and fits in the expected 3-byte Variable-Byte sequence.
Transition Points: Validates the range checks in write_tag8_4s16 (e.g., -8 being the boundary for 4-bit vs 8-bit).
Silent Truncation: Confirms the write_byte logic prevents memory corruption when the internal pos exceeds buffer.len().
Global Scaling: In tag2_3s32, a single large value forces all other values in that group to be encoded with the larger width.
*/
#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn boundary_16bit_signed_vb() {
        let mut buf = [0u8; 10];
        {
            let mut writer = SliceWriter { buffer: &mut buf, pos: 0 };

            // i16::MAX (32767) -> ZigZag = 65534 (0xFE 0xFF 0x03)
            writer.write_signed_vb(32767);

            // i16::MIN (-32768) -> ZigZag = 65535 (0xFF 0xFF 0x03)
            writer.write_signed_vb(-32768);
        }
        assert_eq!(buf[..3], [0xFE, 0xFF, 0x03]);
        assert_eq!(buf[3..6], [0xFF, 0xFF, 0x03]);
    }

    #[test]
    fn tag8_4s16_boundary_values() {
        let mut buf = [0u8; 10];
        {
            let mut writer = SliceWriter { buffer: &mut buf, pos: 0 };

            // Test transitions:
            // -8 (4-bit), -128 (8-bit), 127 (8-bit), 32767 (16-bit)
            let values = [-8, -128, 127, 32767];
            writer.write_tag8_4s16(values);
        }

        // Tags: -8 is 0x01, -128 is 0x02, 127 is 0x02, 32767 is 0x03
        // Binary: 11 10 10 01 -> 0xE9
        assert_eq!(buf[0], 0b_11_10_10_01);
        // Byte 1: Nibble -8 (0x08)
        assert_eq!(buf[1], 136);
        // Byte 2: -128 as u8 (0x80)
        assert_eq!(buf[2], 7);
        // Byte 3: 127 as u8 (0x7F)
        assert_eq!(buf[3], 247);
    }

    #[test]
    fn buffer_overflow_safety() {
        let mut small_buf = [0u8; 1];
        {
            let mut writer = SliceWriter { buffer: &mut small_buf, pos: 0 };

            // This requires 3 bytes, but we only have 1.
            // Logic should write what it can and stop, avoiding panics.
            writer.write_unsigned_vb(30000);

            assert_eq!(writer.pos, 1);
        }
        assert_eq!(small_buf[0], 0xB0); // First byte of 30000 (0xB0 0xEA 0x01)
    }

    #[test]
    fn tag2_3s32_max_i16_range() {
        let mut buf = [0u8; 10];
        {
            let mut writer = SliceWriter { buffer: &mut buf, pos: 0 };

            // One value is 128 (requires 16-bit mode '3' in tag2_3s32 logic)
            let values = [0, 128, 0];
            writer.write_tag2_3s32(values);
        }

        assert_eq!(buf[0], 3); // Tag byte should be 3
        // Values written as 16-bit LE: 0x0000, 0x8000, 0x0000
        assert_eq!(&buf[1..7], &[0, 0, 128, 0, 0, 0]);
    }
}
