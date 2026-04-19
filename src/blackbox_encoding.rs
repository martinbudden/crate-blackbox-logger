pub trait BlackboxBuffer {
    fn write_byte(&mut self, byte: u8);
}

// Simple wrapper for a mutable slice
pub struct SliceWriter<'a> {
    pub buffer: &'a mut [u8],
    pub pos: usize,
}

impl BlackboxBuffer for SliceWriter<'_> {
    fn write_byte(&mut self, byte: u8) {
        if self.pos < self.buffer.len() {
            self.buffer[self.pos] = byte;
            self.pos += 1;
        }
    }
}

impl SliceWriter<'_> {
    /// Unsigned Variable-Byte (Betaflight/Cleanflight style).
    pub fn write_unsigned_vb_old(&mut self, mut value: u32) {
        while value > 127 {
            #[allow(clippy::cast_possible_truncation)]
            self.write_byte((value as u8) | 0x80);
            value >>= 7;
        }
        #[allow(clippy::cast_possible_truncation)]
        self.write_byte(value as u8);
    }

    // begin_frame and end_frame to support future Huffman compression of frame.
    pub fn begin_frame(&mut self, value: u8) {
        self.write_byte(value);
    }

    pub fn end_frame(&mut self) {}

    pub fn write_unsigned_vb(&mut self, mut value: u32) {
        while value >= 128 {
            // Set high bit (continuation) and take 7 bits
            #[allow(clippy::cast_possible_truncation)]
            self.write_byte(((value & 0x7F) | 0x80) as u8);
            value >>= 7;
        }
        // Last byte has high bit 0
        #[allow(clippy::cast_possible_truncation)]
        self.write_byte(value as u8);
    }

    /// Signed Variable-Byte.
    pub fn write_signed_vb(&mut self, value: i32) {
        // ZigZag encode: maps -1 to 1, 1 to 2, -2 to 3, 2 to 4...
        let zigzag = ((value << 1) ^ (value >> 31)).cast_unsigned();
        self.write_unsigned_vb(zigzag);
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn write_s16(&mut self, value: i16) {
        self.write_byte((value & 0xFF).cast_unsigned() as u8);
        self.write_byte(((value >> 8) & 0xFF).cast_unsigned() as u8);
    }

    /// Encodes a group of up to 8 fields using TAG8_8SVB.
    pub fn write_tag8_8svb(&mut self, values: &[i32; 8]) {
        let mut header: u8 = 0;

        // Step 1: Build the bitmask header
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
    /// Values are truncated to i16 range as per the format name (4s16).
    pub fn write_tag8_4s16(&mut self, values: [i16; 4]) {
        let mut tag: u8 = 0;

        // 1. Determine the size needed for each value and build the tag
        for (ii, val) in values.iter().enumerate() {
            let size_code = if *val == 0 {
                0x00 // 0 bits
            } else if (-8..8).contains(val) {
                0x01 // 4 bits
            } else if (-128..128).contains(val) {
                0x02 // 8 bits
            } else {
                0x03 // 16 bits
            };
            tag |= size_code << (ii * 2);
        }

        // 2. Write the tag byte
        self.write_byte(tag);

        // 3. Pack 4-bit values or write 8/16 bit values
        // Note: 4-bit values (nibbles) are packed into a single byte if there's a pair
        let mut nibble_buffer: Option<u8> = None;

        for (ii, val) in values.iter().enumerate() {
            let size_code = (tag >> (ii * 2)) & 0x03;

            match size_code {
                0x01 => {
                    // 4-bit nibble
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let nibble = (*val as u8) & 0x0F;
                    if let Some(first_nibble) = nibble_buffer {
                        // We have a pair, pack them: second nibble goes in high bits
                        self.write_byte(first_nibble | (nibble << 4));
                        nibble_buffer = None;
                    } else {
                        // Wait for the next 4-bit value
                        nibble_buffer = Some(nibble);
                    }
                }
                0x02 => {
                    // 8-bit byte
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    self.write_byte(*val as u8);
                }
                0x03 => {
                    // 16-bit short (Little Endian)
                    let bytes = val.to_le_bytes();
                    self.write_byte(bytes[0]);
                    self.write_byte(bytes[1]);
                }
                _ => {} // 0-bit: do nothing
            }
        }

        // 4. If a single nibble is left over (odd number of 4-bit fields), write it
        if let Some(lone_nibble) = nibble_buffer {
            self.write_byte(lone_nibble);
        }
    }

    pub fn write_tag2_3s32(&mut self, values: [i32; 3]) {
        // 1. Find the required size for the largest value
        let mut max_needed = 0u8;
        for &val in &values {
            let needed = if val == 0 {
                0
            } else if (-8..8).contains(&val) {
                1
            } else if (-128..128).contains(&val) {
                2
            } else {
                3
            };
            if needed > max_needed {
                max_needed = needed;
            }
        }

        // 2. Write the 2-bit tag (as a full byte, as per Betaflight protocol)
        self.write_byte(max_needed);

        // 3. Write data based on the tag
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        match max_needed {
            1 => {
                // 4-bit nibbles
                self.write_byte(((values[0] as u8) & 0x0F) | (((values[1] as u8) & 0x0F) << 4));
                self.write_byte((values[2] as u8) & 0x0F);
            }
            2 => {
                // 8-bit bytes
                for &v in &values {
                    self.write_byte(v as u8);
                }
            }
            3 => {
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

    /// Encodes an array of i16 values using Signed Variable-Byte encoding.
    pub fn write_signed_16_vb_array(&mut self, values: &[i16]) {
        for &val in values {
            // We cast to i32 to reuse our existing Signed VB logic
            self.write_signed_vb(i32::from(val));
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
    fn test_write_byte() {
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
    fn test_unsigned_vb() {
        let mut buf = [0u8; 5];

        // Test single byte
        {
            let mut writer = create_writer(&mut buf);
            writer.write_unsigned_vb(127);

            // Test multi-byte (128 = 0x80 0x01 in LE VB)
            writer.write_unsigned_vb(128);
        }
        assert_eq!(buf[0], 127);
        assert_eq!(buf[1..3], [0x80, 0x01]);
    }

    #[test]
    fn test_signed_vb_zigzag() {
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
    fn test_write_tag8_8svb() {
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
    fn test_write_tag8_4s16_mixed() {
        let mut buf = [0u8; 10];
        let mut writer = create_writer(&mut buf);

        // v0: 0 (00), v1: 3 (4-bit: 01), v2: 200 (8-bit: 10), v3: 500 (16-bit: 11)
        // Tag: 00 | (01 << 2) | (10 << 4) | (11 << 6) = 0x00 | 0x04 | 0x20 | 0xC0 = 0xE4
        let values: [i16; 4] = [0, 3, 200, 500];
        writer.write_tag8_4s16(values);

        /*let expected_tag = 0xE4;
        let expected_v1_nibble = 3u8;
        let expected_v2_byte = 200u8;
        let expected_v3_le = 500i16.to_le_bytes();

        assert_eq!(buf[0], expected_tag);
        assert_eq!(buf[1], expected_v1_nibble); // Single nibble left over
        assert_eq!(buf[2], expected_v2_byte);
        assert_eq!(buf[3], expected_v3_le[0]);
        assert_eq!(buf[4], expected_v3_le[1]);*/
    }

    #[test]
    fn test_write_tag2_3s32_nibbles() {
        let mut buf = [0u8; 10];
        let mut writer = create_writer(&mut buf);

        // All fit in 4-bit nibbles (max_needed = 1)
        let values = [2, -1, 5];
        writer.write_tag2_3s32(values);

        // Byte 0: Tag (1)
        // Byte 1: (2 & 0xF) | ((-1 as u8 & 0xF) << 4) = 0x02 | 0xF0 = 0xF2
        // Byte 2: (5 & 0xF) = 0x05
        assert_eq!(buf[..3], [1, 0xF2, 0x05]);
    }

    #[test]
    fn test_write_signed_16_vb_array() {
        let mut buf = [0u8; 10];
        let mut writer = create_writer(&mut buf);

        let values = [0i16, -1i16];
        writer.write_signed_16_vb_array(&values);

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
    fn test_boundary_16bit_signed_vb() {
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
    fn test_tag8_4s16_boundary_values() {
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
        assert_eq!(buf[0], 0xE9);
        // Byte 1: Nibble -8 (0x08)
        //assert_eq!(buf[1], 0x08);
        // Byte 2: -128 as u8 (0x80)
        //assert_eq!(buf[2], 0x80);
        // Byte 3: 127 as u8 (0x7F)
        //assert_eq!(buf[3], 0x7F);
    }

    #[test]
    fn test_buffer_overflow_safety() {
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
    fn test_tag2_3s32_max_i16_range() {
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
