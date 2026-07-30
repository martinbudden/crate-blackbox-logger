#![cfg(feature = "huffman")]
use crate::huffman_table::HUFFMAN_TABLE;

#[derive(Debug, Default, PartialEq)]
pub struct HuffmanEncoder<'a> {
    output: &'a mut [u8],
    write_idx: usize,
    bit_buffer: u32,
    bit_count: u32,
}

impl<'a> HuffmanEncoder<'a> {
    pub fn new(output: &'a mut [u8]) -> Self {
        Self {
            output,
            // Start at index 1 to reserve index 0 for the uncompressed length of the input stream
            write_idx: 1,
            bit_buffer: 0,
            bit_count: 0,
        }
    }

    /// Internal helper to push bits into the slice.
    /// Expects left-aligned bits from a u16 code.
    #[inline]
    fn write_bits(&mut self, code: u16, len: u32) -> Result<(), ()> {
        // Shift code up to the top of the u32 accumulator,
        // then align it to the current bit cursor position.
        let code_u32 = u32::from(code) << 16;
        self.bit_buffer |= code_u32 >> self.bit_count;
        self.bit_count += len;

        // Drain full bytes out of the top of the accumulator
        while self.bit_count >= 8 {
            if self.write_idx >= self.output.len() {
                return Err(());
            }

            // Extract the highest 8 bits
            self.output[self.write_idx] = (self.bit_buffer >> 24) as u8;
            self.write_idx += 1;

            // Shift accumulator left and update bit count
            self.bit_buffer <<= 8;
            self.bit_count -= 8;
        }
        Ok(())
    }

    /// O(1) compression routine per byte.
    /// Prepends a u8 uncompressed length byte to the output slice.
    pub fn compress(mut self, input: &[u8]) -> Result<usize, ()> {
        // Enforce that the stream is under 255 bytes
        if input.len() > 255 {
            return Err(());
        }

        // Check if the buffer can even hold the 1-byte length header
        if self.output.is_empty() {
            return Err(());
        }

        // Process the input bytes
        for &byte in input {
            let huff_code = HUFFMAN_TABLE[byte as usize];
            self.write_bits(huff_code.code, u32::from(huff_code.len))?;
        }

        // Flush remaining partial bits (padded with trailing zeros)
        if self.bit_count > 0 {
            if self.write_idx >= self.output.len() {
                return Err(());
            }
            self.output[self.write_idx] = (self.bit_buffer >> 24) as u8;
            self.write_idx += 1;
        }

        // Write the input length into the reserved first byte
        #[allow(clippy::cast_possible_truncation)]
        {
            self.output[0] = input.len() as u8;
        }

        // Return total bytes written (length byte + compressed data payload)
        Ok(self.write_idx)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_partial<T: Sized + Send + Sync + Unpin + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_partial::<HuffmanEncoder>();
    }

    // 0x0 11
    // 0x1 101
    // 0x2 1001
    // 0x3 10001
    // 0x4 10000
    // 0x5 011101

    #[test]
    fn test_single_byte_zero() {
        let input = [0u8];
        let mut output = [0u8; 16];

        let writer = HuffmanEncoder::new(&mut output);
        let result = writer.compress(&input);

        // Expected compression stream:
        // '0' symbol:  11 (2 bits)
        // Stream:       11000000 (padded with trailing 0s) -> [1, 0xC0]
        assert_eq!(result, Ok(2)); // length of output stream, including length byte, is 2
        // the length byte is the length of the input stream
        assert_eq!(&output[..2], &[1, 0xC0]);
    }

    #[test]
    fn test_sequence_zero_to_four() {
        let input = [0u8, 1u8, 2u8, 3u8, 4u8];
        let mut output = [0u8; 16];

        let writer = HuffmanEncoder::new(&mut output);
        let result = writer.compress(&input);

        // Expected compression stream:
        // '0':   11       (2 bits)
        // '1':   101      (3 bits)
        // '2':   1001     (4 bits)
        // '3':   10001    (5 bits)
        // '4':   10000    (5 bits)
        // Stream: 11101100 11000110 00 000000 -> [5, 0xEC, 0xC6, 0x00]
        assert_eq!(result, Ok(4)); // length of output stream, including length byte, is 4
        // the length byte is the length of the input stream
        assert_eq!(&output[..4], &[5, 0xEC, 0xC6, 0x00]);
    }

    #[test]
    fn test_buffer_overflow_error() {
        let input = [0u8, 1u8, 2u8, 3u8, 4u8];
        let mut tiny_output = [0u8; 2]; // Too small for result

        let writer = HuffmanEncoder::new(&mut tiny_output);
        let result = writer.compress(&input);

        assert_eq!(result, Err(()));
    }
}
